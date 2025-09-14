use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub struct ExitCommand;

#[async_trait::async_trait]
impl BuiltinCommand for ExitCommand {
    async fn execute(
        &self,
        command: &ParsedCommand,
        _current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
        _parser: &mut crate::core::parser::Parser,
    ) -> Result<i32> {
        let _exit_code = if command.args.is_empty() {
            0
        } else {
            command.args[0].parse::<i32>().unwrap_or(0)
        };

        // Special exit code to signal the shell to exit
        Ok(130)
    }

    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Exit the shell"
    }

    fn usage(&self) -> &'static str {
        "exit [status]\n  status  Exit status (default: 0)"
    }
}