use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub struct EnvCommand;

#[async_trait::async_trait]
impl BuiltinCommand for EnvCommand {
    async fn execute(
        &self,
        command: &ParsedCommand,
        _current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
        _parser: &mut crate::core::parser::Parser,
    ) -> Result<i32> {
        if command.args.is_empty() {
            let mut env_vars: Vec<_> = std::env::vars().collect();
            env_vars.sort_by(|a, b| a.0.cmp(&b.0));

            for (key, value) in env_vars {
                println!("{}={}", key, value);
            }
        } else {
            for arg in &command.args {
                if let Some(eq_pos) = arg.find('=') {
                    let (key, value) = arg.split_at(eq_pos);
                    let value = &value[1..]; // Skip the '=' character
                    std::env::set_var(key, value);
                    println!("Set {}={}", key, value);
                } else {
                    match std::env::var(arg) {
                        Ok(value) => println!("{}={}", arg, value),
                        Err(_) => {
                            eprintln!("env: {}: not found", arg);
                            return Ok(1);
                        }
                    }
                }
            }
        }

        Ok(0)
    }

    fn name(&self) -> &'static str {
        "env"
    }

    fn description(&self) -> &'static str {
        "Display or set environment variables"
    }

    fn usage(&self) -> &'static str {
        "env [VAR=value ...] [variable ...]\n  VAR=value  Set environment variable\n  variable   Display specific variable"
    }
}