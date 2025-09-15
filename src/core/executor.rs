use crate::core::parser::ParsedCommand;
use crate::builtins::{self, BuiltinCommand};
use anyhow::Result;
use log::debug;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::process::{Child, Command as TokioCommand};
use tokio::fs::File;
use tokio::signal;

#[cfg(windows)]
use winapi::um::{
    handleapi::CloseHandle,
    jobapi2::{CreateJobObjectW, AssignProcessToJobObject, SetInformationJobObject, JobObjectExtendedLimitInformation},
    processthreadsapi::TerminateJobObject,
    winnt::{HANDLE, JOB_OBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE},
    winbase::CREATE_NEW_PROCESS_GROUP,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

#[cfg(windows)]
pub struct WindowsProcessManager {
    job_handle: Option<winapi::um::winnt::HANDLE>,
}

#[cfg(windows)]
impl WindowsProcessManager {
    pub fn new() -> Self {
        Self { job_handle: None }
    }

    pub fn create_job(&mut self) -> Result<()> {
        unsafe {
            let job_handle = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
            if job_handle.is_null() {
                return Err(anyhow::anyhow!("Failed to create job object"));
            }

            // Set up job to kill all processes when the job handle is closed
            let mut job_info: JOB_OBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            job_info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let result = SetInformationJobObject(
                job_handle,
                JobObjectExtendedLimitInformation,
                &job_info as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<JOB_OBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );

            if result == 0 {
                CloseHandle(job_handle);
                return Err(anyhow::anyhow!("Failed to set job information"));
            }

            self.job_handle = Some(job_handle);
            Ok(())
        }
    }

    pub fn assign_process(&self, process_handle: winapi::um::winnt::HANDLE) -> Result<()> {
        if let Some(job_handle) = self.job_handle {
            unsafe {
                if AssignProcessToJobObject(job_handle, process_handle) == 0 {
                    return Err(anyhow::anyhow!("Failed to assign process to job"));
                }
            }
        }
        Ok(())
    }

    pub fn terminate_job(&self) -> Result<()> {
        if let Some(job_handle) = self.job_handle {
            unsafe {
                use winapi::um::processthreadsapi::TerminateJobObject;
                if TerminateJobObject(job_handle, 130) == 0 {
                    return Err(anyhow::anyhow!("Failed to terminate job"));
                }
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
impl Drop for WindowsProcessManager {
    fn drop(&mut self) {
        if let Some(job_handle) = self.job_handle {
            unsafe {
                CloseHandle(job_handle);
            }
        }
    }
}

pub struct Executor {
	background_processes: HashMap<u32, Child>,
	interrupt_flag: Arc<AtomicBool>,
	#[cfg(windows)]
	process_manager: WindowsProcessManager,
}

impl Executor {
	pub fn new() -> Self {
		let interrupt_flag = Arc::new(AtomicBool::new(false));
		Self {
			background_processes: HashMap::new(),
			interrupt_flag,
			#[cfg(windows)]
			process_manager: WindowsProcessManager::new(),
		}
	}

	fn resolve_program_path(&self, program_name: &str) -> Option<PathBuf> {
		debug!("Resolving program path for: '{}'", program_name);
		if program_name.contains('/') || program_name.contains('\\') {
			let path = Path::new(program_name);
			return if path.exists() { Some(path.to_path_buf()) } else { None };
		}
		if let Ok(path_var) = std::env::var("PATH") {
			#[cfg(windows)]
			let path_separator = ";";
			#[cfg(not(windows))]
			let path_separator = ":";
			#[cfg(windows)]
			let executable_extensions = vec!["exe", "bat", "cmd", "com"];
			for path_dir in path_var.split(path_separator) {
				if path_dir.is_empty() { continue; }
				let dir_path = Path::new(path_dir);
				if !dir_path.exists() || !dir_path.is_dir() { continue; }
				#[cfg(windows)] {
					let exe_candidate = dir_path.join(format!("{}.exe", program_name));
					if exe_candidate.exists() && exe_candidate.is_file() {
						return Some(exe_candidate);
					}
					for ext in &executable_extensions {
						if ext == &"exe" { continue; }
						let candidate_with_ext = dir_path.join(format!("{}.{}", program_name, ext));
						if candidate_with_ext.exists() && candidate_with_ext.is_file() {
							return Some(candidate_with_ext);
						}
					}
				}
				#[cfg(not(windows))] {
					let candidate = dir_path.join(program_name);
					if candidate.exists() && candidate.is_file() {
						return Some(candidate);
					}
				}
			}
		}
		None
	}

	pub async fn execute(&mut self, command: ParsedCommand, current_dir: &mut PathBuf, parser: &mut crate::core::parser::Parser) -> Result<i32> {
		debug!("Executing command: {:?}", command);
		if let Some(builtin) = builtins::get_builtin(&command.program) {
			return self.execute_builtin(builtin, &command, current_dir, parser).await;
		}
		if command.pipes.is_empty() {
			self.execute_single_command(command, current_dir).await
		} else {
			self.execute_pipeline(command, current_dir).await
		}
	}

	async fn execute_builtin(
		&mut self,
		builtin: Box<dyn BuiltinCommand>,
		command: &ParsedCommand,
		current_dir: &mut PathBuf,
		parser: &mut crate::core::parser::Parser,
	) -> Result<i32> {
		builtin.execute(command, current_dir, &mut self.background_processes, parser).await
	}

	async fn execute_single_command(&mut self, command: ParsedCommand, current_dir: &PathBuf) -> Result<i32> {
		let program_path = if let Some(resolved_path) = self.resolve_program_path(&command.program) {
			resolved_path
		} else {
			return Err(anyhow::anyhow!("program not found: {}", command.program));
		};

		let mut cmd = {
			#[cfg(windows)] {
				if let Some(ext) = program_path.extension() {
					let ext = ext.to_string_lossy().to_lowercase();
					if ext == "bat" || ext == "cmd" {
						let mut cmd = TokioCommand::new("cmd");
						cmd.arg("/c");
						cmd.arg(&program_path);
						cmd.args(&command.args);
						// Create a new process group so we can send Ctrl+C to the entire group
						cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
						cmd
					} else {
						let mut cmd = TokioCommand::new(&program_path);
						cmd.args(&command.args);
						// Create a new process group so we can send Ctrl+C to the entire group
						cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
						cmd
					}
				} else {
					let mut cmd = TokioCommand::new(&program_path);
					cmd.args(&command.args);
					// Create a new process group so we can send Ctrl+C to the entire group
					cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
					cmd
				}
			}
			#[cfg(not(windows))] {
				let mut cmd = TokioCommand::new(&program_path);
				cmd.args(&command.args);
				cmd
			}
		};

		cmd.current_dir(current_dir);
		for (key, value) in &command.environment {
			cmd.env(key, value);
		}

		if let Some(input_file) = &command.input_redirect {
			let file = File::open(input_file).await?;
			cmd.stdin(Stdio::from(file.into_std().await));
		}

		if let Some(output_file) = &command.output_redirect {
			let file = File::create(output_file).await?;
			cmd.stdout(Stdio::from(file.into_std().await));
		} else if let Some(append_file) = &command.append_redirect {
			let file = tokio::fs::OpenOptions::new()
				.create(true)
				.append(true)
				.open(append_file)
				.await?;
			cmd.stdout(Stdio::from(file.into_std().await));
		}

		if command.background {
			let child = cmd.spawn()?;
			let pid = child.id().unwrap_or(0);
			self.background_processes.insert(pid, child);
			println!("[{}] {}", self.background_processes.len(), pid);
			return Ok(0);
		}

		// Create job object on Windows for better process group management
		#[cfg(windows)]
		{
			self.process_manager.create_job()?;
		}

		let mut child = cmd.spawn()?;
		let child_id = child.id().unwrap_or(0);

		// Assign the process to the job on Windows
		#[cfg(windows)]
		{
			use std::os::windows::io::AsRawHandle;
			if let Some(handle) = child.raw_handle() {
				let _ = self.process_manager.assign_process(handle as winapi::um::winnt::HANDLE);
			}
		}

		let res = tokio::select! {
			status = child.wait() => {
				status?.code().unwrap_or(-1)
			}
			_ = signal::ctrl_c() => {
				debug!("Ctrl+C received, terminating child process {}", child_id);
				
				#[cfg(unix)] {
					use nix::sys::signal::{kill, Signal};
					use nix::unistd::Pid;
					let _ = kill(Pid::from_raw(child_id as i32), Signal::SIGINT);
				}
				
				#[cfg(windows)] {
					// Use the job object to terminate the entire process tree
					if let Err(e) = self.process_manager.terminate_job() {
						debug!("Failed to terminate via job object: {}, falling back to kill", e);
						let _ = child.kill().await;
					}
				}
				
				let _ = child.wait().await;
				130
			}
		};

		Ok(res)
	}

	async fn execute_pipeline(&mut self, command: ParsedCommand, current_dir: &PathBuf) -> Result<i32> {
		if command.pipes.is_empty() {
			return self.execute_single_command(command, current_dir).await;
		}
		let mut commands = vec![command.clone()];
		commands.extend(command.pipes.clone());
		let mut processes = Vec::new();
		let mut previous_stdout = None;

		// Create job object on Windows for better process group management
		#[cfg(windows)]
		{
			self.process_manager.create_job()?;
		}

		for (i, pipeline_cmd) in commands.iter().enumerate() {
			let program_path = if let Some(resolved_path) = self.resolve_program_path(&pipeline_cmd.program) {
				resolved_path
			} else {
				return Err(anyhow::anyhow!("program not found: {}", pipeline_cmd.program));
			};
			let mut tokio_cmd = {
				#[cfg(windows)] {
					if let Some(ext) = program_path.extension() {
						let ext = ext.to_string_lossy().to_lowercase();
						if ext == "bat" || ext == "cmd" {
							let mut cmd = TokioCommand::new("cmd");
							cmd.arg("/c");
							cmd.arg(&program_path);
							cmd.args(&pipeline_cmd.args);
							// Create a new process group for pipeline commands
							cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
							cmd
						} else {
							let mut cmd = TokioCommand::new(&program_path);
							cmd.args(&pipeline_cmd.args);
							// Create a new process group for pipeline commands
							cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
							cmd
						}
					} else {
						let mut cmd = TokioCommand::new(&program_path);
						cmd.args(&pipeline_cmd.args);
						// Create a new process group for pipeline commands
						cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
						cmd
					}
				}
				#[cfg(not(windows))] {
					let mut cmd = TokioCommand::new(&program_path);
					cmd.args(&pipeline_cmd.args);
					cmd
				}
			};
			tokio_cmd.current_dir(current_dir);
			for (key, value) in &pipeline_cmd.environment {
				tokio_cmd.env(key, value);
			}
			if i == 0 {
				if let Some(input_file) = &pipeline_cmd.input_redirect {
					let file = File::open(input_file).await?;
					tokio_cmd.stdin(Stdio::from(file.into_std().await));
				} else {
					tokio_cmd.stdin(Stdio::inherit());
				}
			} else {
				tokio_cmd.stdin(previous_stdout.take().unwrap_or(Stdio::piped()));
			}
			if i == commands.len() - 1 {
				if let Some(output_file) = &pipeline_cmd.output_redirect {
					let file = File::create(output_file).await?;
					tokio_cmd.stdout(Stdio::from(file.into_std().await));
				} else if let Some(append_file) = &pipeline_cmd.append_redirect {
					let file = tokio::fs::OpenOptions::new()
						.create(true)
						.append(true)
						.open(append_file)
						.await?;
					tokio_cmd.stdout(Stdio::from(file.into_std().await));
				} else {
					tokio_cmd.stdout(Stdio::inherit());
				}
			} else {
				tokio_cmd.stdout(Stdio::piped());
			}
			tokio_cmd.stderr(Stdio::inherit());
			let mut child = tokio_cmd.spawn()?;

			// Assign processes to job on Windows
			#[cfg(windows)]
			{
				use std::os::windows::io::AsRawHandle;
				if let Some(handle) = child.raw_handle() {
					let _ = self.process_manager.assign_process(handle as winapi::um::winnt::HANDLE);
				}
			}

			if i < commands.len() - 1 {
				if let Some(_stdout) = child.stdout.take() {
					previous_stdout = Some(Stdio::piped());
				}
			}
			processes.push(child);
		}

		// Wait for all processes with Ctrl+C handling
		let mut exit_code = 0;
		let process_ids: Vec<Option<u32>> = processes.iter().map(|p| p.id()).collect();
		
		let pipeline_fut = async {
			for mut process in processes {
				let status = process.wait().await?;
				exit_code = status.code().unwrap_or(-1);
			}
			Ok::<i32, anyhow::Error>(exit_code)
		};

		let res = tokio::select! {
			result = pipeline_fut => {
				result?
			}
			_ = signal::ctrl_c() => {
				debug!("Ctrl+C received, terminating pipeline");
				
				#[cfg(windows)] {
					// Use the job object to terminate the entire process tree
					if let Err(e) = self.process_manager.terminate_job() {
						debug!("Failed to terminate via job object: {}", e);
					}
				}
				
				#[cfg(unix)] {
					// On Unix, we should try to kill all processes by their PIDs
					use nix::sys::signal::{kill, Signal};
					use nix::unistd::Pid;
					for pid in process_ids.iter().filter_map(|&p| p) {
						let _ = kill(Pid::from_raw(pid as i32), Signal::SIGINT);
					}
				}
				
				130
			}
		};

		Ok(res)
	}

	pub async fn cleanup_background_processes(&mut self) -> Result<()> {
		let mut completed = Vec::new();
		for (pid, child) in &mut self.background_processes {
			if let Ok(Some(status)) = child.try_wait() {
				println!("[{}] Done    {}", pid, status.code().unwrap_or(-1));
				completed.push(*pid);
			}
		}
		for pid in completed {
			self.background_processes.remove(&pid);
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
    include!("executor_tests.rs");
}
