use super::{BuiltinCommand, list_builtins, get_builtin};
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub struct HelpCommand;

#[async_trait::async_trait]
impl BuiltinCommand for HelpCommand {
    async fn execute(
        &self,
        command: &ParsedCommand,
        _current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
    ) -> Result<i32> {
        if command.args.is_empty() {
            println!("Flex-SH - A high-performance, modern system shell");
            println!();
            println!("Built-in commands:");

            let builtins = list_builtins();
            for builtin_name in builtins {
                if let Some(builtin) = get_builtin(builtin_name) {
                    println!("  {:10} - {}", builtin.name(), builtin.description());
                }
            }
            println!();
            println!("Use 'help <command>' for detailed information about a specific command.");
        } else {
            let command_name = &command.args[0];
            if let Some(builtin) = get_builtin(command_name) {
                println!("{} - {}", builtin.name(), builtin.description());
                println!();
                println!("Usage:");
                println!("{}", builtin.usage());
            } else {
                println!("Unknown command: {}", command_name);
                return Ok(1);
            }
        }

        Ok(0)
    }

    fn name(&self) -> &'static str {
        "help"
    }

    fn description(&self) -> &'static str {
        "Display help information"
    }

    fn usage(&self) -> &'static str {
        "help [command]\n  command  Show help for a specific command"
    }
}