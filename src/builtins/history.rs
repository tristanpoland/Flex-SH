use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub struct HistoryCommand;

#[async_trait::async_trait]
impl BuiltinCommand for HistoryCommand {
    async fn execute(
        &self,
        command: &ParsedCommand,
        _current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
    ) -> Result<i32> {
        // TODO: Integrate with actual history system
        if command.args.is_empty() {
            println!("history: command not yet fully implemented");
        } else {
            match command.args[0].as_str() {
                "-c" => {
                    println!("history: cleared");
                }
                _ => {
                    println!("history: unknown option: {}", command.args[0]);
                    return Ok(1);
                }
            }
        }

        Ok(0)
    }

    fn name(&self) -> &'static str {
        "history"
    }

    fn description(&self) -> &'static str {
        "Display or manipulate command history"
    }

    fn usage(&self) -> &'static str {
        "history [-c]\n  -c  Clear history"
    }
}