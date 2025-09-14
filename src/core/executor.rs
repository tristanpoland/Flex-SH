use crate::core::parser::ParsedCommand;
use crate::builtins::{self, BuiltinCommand};
use anyhow::Result;
use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command as TokioCommand};
use tokio::fs::File;

pub struct Executor {
    background_processes: HashMap<u32, Child>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            background_processes: HashMap::new(),
        }
    }

    pub async fn execute(&mut self, command: ParsedCommand, current_dir: &mut PathBuf) -> Result<i32> {
        debug!("Executing command: {:?}", command);

        if let Some(builtin) = builtins::get_builtin(&command.program) {
            return self.execute_builtin(builtin, &command, current_dir).await;
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
    ) -> Result<i32> {
        debug!("Executing builtin: {}", command.program);
        builtin.execute(command, current_dir, &mut self.background_processes).await
    }

    async fn execute_single_command(&mut self, command: ParsedCommand, current_dir: &PathBuf) -> Result<i32> {
        let mut cmd = TokioCommand::new(&command.program);
        cmd.args(&command.args);
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
        let status = child.wait().await?;

        Ok(status.code().unwrap_or(-1))
    }

    async fn execute_pipeline(&mut self, mut command: ParsedCommand, current_dir: &PathBuf) -> Result<i32> {
        if command.pipes.is_empty() {
            return self.execute_single_command(command, current_dir).await;
        }

        let mut commands = vec![command.clone()];
        commands.extend(command.pipes.clone());

        let mut processes = Vec::new();
        let mut previous_stdout = None;

        for (i, cmd) in commands.iter().enumerate() {
            let mut tokio_cmd = TokioCommand::new(&cmd.program);
            tokio_cmd.args(&cmd.args);
            tokio_cmd.current_dir(current_dir);

            for (key, value) in &cmd.environment {
                tokio_cmd.env(key, value);
            }

            if i == 0 {
                if let Some(input_file) = &cmd.input_redirect {
                    let file = File::open(input_file).await?;
                    tokio_cmd.stdin(Stdio::from(file.into_std().await));
                } else {
                    tokio_cmd.stdin(Stdio::piped());
                }
            } else {
                tokio_cmd.stdin(previous_stdout.take().unwrap_or(Stdio::piped()));
            }

            if i == commands.len() - 1 {
                if let Some(output_file) = &cmd.output_redirect {
                    let file = File::create(output_file).await?;
                    tokio_cmd.stdout(Stdio::from(file.into_std().await));
                } else if let Some(append_file) = &cmd.append_redirect {
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

            let mut child = tokio_cmd.spawn()?;

            if i < commands.len() - 1 {
                if let Some(_stdout) = child.stdout.take() {
                    // Simplified - just pipe to next process
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