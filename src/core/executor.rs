use crate::core::parser::ParsedCommand;
use crate::builtins::{self, BuiltinCommand};
use anyhow::Result;
use log::debug;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use ctrlc;
use tokio::process::{Child, Command as TokioCommand};
use tokio::fs::File;

pub struct Executor {
	background_processes: HashMap<u32, Child>,
	interrupt_flag: Arc<AtomicBool>,
}

impl Executor {
	pub fn new() -> Self {
		let interrupt_flag = Arc::new(AtomicBool::new(false));
		let flag_clone = interrupt_flag.clone();
		ctrlc::set_handler(move || {
			if !flag_clone.load(Ordering::SeqCst) {
				flag_clone.store(true, Ordering::SeqCst);
			} else {
				std::process::exit(130);
			}
		}).expect("Error setting Ctrl-C handler");
		Self {
			background_processes: HashMap::new(),
			interrupt_flag,
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
		if self.interrupt_flag.load(Ordering::SeqCst) {
			self.interrupt_flag.store(false, Ordering::SeqCst);
			return Ok(130);
		}
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
						cmd
					} else {
						let mut cmd = TokioCommand::new(&program_path);
						cmd.args(&command.args);
						cmd
					}
				} else {
					let mut cmd = TokioCommand::new(&program_path);
					cmd.args(&command.args);
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
		let mut child = cmd.spawn()?;
		let interrupt_flag = self.interrupt_flag.clone();
		let child_id = child.id().unwrap_or(0);
		let res = tokio::select! {
			status = child.wait() => {
				status?.code().unwrap_or(-1)
			}
			_ = async {
				while !interrupt_flag.load(Ordering::SeqCst) {
					tokio::time::sleep(std::time::Duration::from_millis(100)).await;
				}
			} => {
				#[cfg(unix)] {
					use nix::sys::signal::{kill, Signal};
					use nix::unistd::Pid;
					let _ = kill(Pid::from_raw(child_id as i32), Signal::SIGINT);
				}
				#[cfg(windows)] {
					let _ = child.kill().await;
				}
				interrupt_flag.store(false, Ordering::SeqCst);
				130
			}
		};
		Ok(res)
	}

	async fn execute_pipeline(&mut self, mut command: ParsedCommand, current_dir: &PathBuf) -> Result<i32> {
		if command.pipes.is_empty() {
			return self.execute_single_command(command, current_dir).await;
		}
		let mut commands = vec![command.clone()];
		commands.extend(command.pipes.clone());
		let mut processes = Vec::new();
		let mut previous_stdout = None;
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
							cmd
						} else {
							let mut cmd = TokioCommand::new(&program_path);
							cmd.args(&pipeline_cmd.args);
							cmd
						}
					} else {
						let mut cmd = TokioCommand::new(&program_path);
						cmd.args(&pipeline_cmd.args);
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
			if i < commands.len() - 1 {
				if let Some(_stdout) = child.stdout.take() {
					previous_stdout = Some(Stdio::piped());
				}
			}
			processes.push(child);
		}
		let mut exit_code = 0;
		for mut process in processes {
			let status = process.wait().await?;
			exit_code = status.code().unwrap_or(-1);
		}
		Ok(exit_code)
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
