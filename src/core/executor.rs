#[cfg(windows)]
#[repr(C)]
pub struct JobObjectExtendedLimitInformation {
	pub basic_limit_information: winapi::um::winnt::JOBOBJECT_BASIC_LIMIT_INFORMATION,
	pub io_info: winapi::um::winnt::IO_COUNTERS,
	pub process_memory_limit: usize,
	pub job_memory_limit: usize,
	pub peak_process_memory_used: usize,
	pub peak_job_memory_used: usize,
}
use portable_pty::{CommandBuilder, PtySize, NativePtySystem};
use portable_pty::PtySystem; // Import the trait for openpty
use std::io::{self, Write, Read};
use crate::core::parser::ParsedCommand;
use crate::builtins::{self, BuiltinCommand};
use anyhow::Result;
use log::debug;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, atomic::AtomicBool};
use tokio::process::{Child, Command as TokioCommand};
use tokio::fs::File;
use tokio::signal;

#[cfg(windows)]
use winapi::um::{
    handleapi::CloseHandle,
	jobapi2::{CreateJobObjectW, AssignProcessToJobObject, SetInformationJobObject},
	winnt::{JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE},
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
			let mut job_info: JobObjectExtendedLimitInformation = std::mem::zeroed();
			job_info.basic_limit_information.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let result = SetInformationJobObject(
                job_handle,
				9, // JobObjectExtendedLimitInformation class
				&job_info as *const _ as *mut std::ffi::c_void,
				std::mem::size_of::<JobObjectExtendedLimitInformation>() as u32,
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
				use winapi::um::jobapi2::TerminateJobObject;
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
			let executable_extensions = vec!["exe", "cmd", "bat", "com"];
			for path_dir in path_var.split(path_separator) {
				if path_dir.is_empty() { continue; }
				let dir_path = Path::new(path_dir);
				if !dir_path.exists() || !dir_path.is_dir() { continue; }
				// Try all extensions in order of preference
				for ext in &executable_extensions {
					let candidate = dir_path.join(format!("{}.{}", program_name, ext));
					if candidate.exists() && candidate.is_file() {
						return Some(candidate);
					}
				}
				// If no extension, check for direct match (scripts, etc.)
				let candidate = dir_path.join(program_name);
				if candidate.exists() && candidate.is_file() {
					return Some(candidate);
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
		// Use PTY for interactive foreground jobs
		if !command.background {
			let pty_system = NativePtySystem::default();
			let pty_pair = pty_system.openpty(PtySize {
				rows: 40,
				if !command.background {
					let pty_system = NativePtySystem::default();
					let pty_pair = pty_system.openpty(PtySize {
						rows: 40,
						cols: 120,
						pixel_width: 0,
						pixel_height: 0,
					})?;

					// On Windows, if the resolved program is a script, spawn via cmd.exe /c
					#[cfg(windows)]
					let resolved_path = self.resolve_program_path(&command.program);
					#[cfg(windows)]
					let (program_to_run, args_to_run) = if let Some(ref path) = resolved_path {
						if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
							match ext.to_ascii_lowercase().as_str() {
								"cmd" | "bat" => ("cmd", vec!["/c", path.to_str().unwrap()]),
								"exe" | "com" => (path.to_str().unwrap(), vec![]),
								_ => (path.to_str().unwrap(), vec![]),
							}
						} else {
							(path.to_str().unwrap(), vec![])
						}
					} else {
						(&command.program, vec![])
					};

					#[cfg(not(windows))]
					let program_to_run = &command.program;
					#[cfg(not(windows))]
					let args_to_run: Vec<&str> = command.args.iter().map(|s| s.as_str()).collect();

					let mut cmd_builder = CommandBuilder::new(program_to_run);
					#[cfg(windows)]
					{
						for arg in &args_to_run {
							cmd_builder.arg(arg);
						}
						for arg in &command.args {
							cmd_builder.arg(arg);
						}
					}
					#[cfg(not(windows))]
					{
						cmd_builder.args(&command.args);
					}
					cmd_builder.cwd(current_dir);
					for (key, value) in &command.environment {
						cmd_builder.env(key, value);
					}
					// TODO: handle input/output redirection for PTY

					let mut child = pty_pair.slave.spawn_command(cmd_builder)?;
					let mut reader = pty_pair.master.try_clone_reader()?;
					let mut writer = pty_pair.master.take_writer()?;

					// Forward PTY output to stdout
					std::thread::spawn(move || {
						let mut buf = [0u8; 4096];
						let mut stdout = io::stdout();
						while let Ok(n) = reader.read(&mut buf) {
							if n == 0 { break; }
							let _ = stdout.write_all(&buf[..n]);
							let _ = stdout.flush();
						}
					});

					// Forward stdin to PTY
					std::thread::spawn(move || {
						let mut buf = [0u8; 4096];
						let mut stdin = io::stdin();
						while let Ok(n) = stdin.read(&mut buf) {
							if n == 0 { break; }
							let _ = writer.write_all(&buf[..n]);
							let _ = writer.flush();
						}
					});

					// Wait for process and handle Ctrl+C
					let res = tokio::select! {
						status = tokio::task::spawn_blocking(move || child.wait().map(|s| {
							// Try to extract exit code from status using Debug output for now
							let dbg = format!("{:?}", s);
							if let Some(start) = dbg.find("Exited(") {
								let code_str = &dbg[start + 7..];
								if let Some(end) = code_str.find(')') {
									if let Ok(code) = code_str[..end].parse::<i32>() {
										return code;
									}
								}
							}
							-1
						})) => {
							status.unwrap_or(-1)
						}
						_ = signal::ctrl_c() => {
							debug!("Ctrl+C received, terminating PTY child");
							#[cfg(unix)] {
								use nix::sys::signal::{kill, Signal};
								use nix::unistd::Pid;
								if let Some(pid) = child.process_id() {
									let _ = kill(Pid::from_raw(pid as i32), Signal::SIGINT);
								}
							}
							#[cfg(windows)] {
								// ConPTY: send Ctrl+C via job object or process group if needed
								let _ = child.kill();
							}
							130
						}
					};
					return Ok(res);
							// Try to kill the process by PID
							use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
							let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
							if handle != std::ptr::null_mut() {
								let _ = unsafe { TerminateProcess(handle, 130) };
							}
						}
					}
					Ok(130 as i32)
				}
			};
			return match res {
				Ok(code) => Ok(code),
				Err(_) => Ok(-1),
			};
		} else {
			// Fallback to old logic for background jobs
			// ...existing code...
			// For now, return an error to avoid type mismatch
			Err(anyhow::anyhow!("Background job execution not implemented"))
		}
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
	// let process_ids: Vec<Option<u32>> = processes.iter().map(|p| p.id()).collect();
		
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
