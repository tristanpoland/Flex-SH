use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub struct CdCommand;

#[async_trait::async_trait]
impl BuiltinCommand for CdCommand {
    async fn execute(
        &self,
        command: &ParsedCommand,
        current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
    ) -> Result<i32> {
        let target_dir = if command.args.is_empty() {
            dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?
        } else {
            let path = &command.args[0];
            if path == "-" {
                std::env::var("OLDPWD")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| current_dir.clone())
            } else {
                let mut target = current_dir.clone();
                target.push(path);
                target.canonicalize().unwrap_or(target)
            }
        };

        if !target_dir.exists() {
            eprintln!("cd: no such file or directory: {}", target_dir.display());
            return Ok(1);
        }

        if !target_dir.is_dir() {
            eprintln!("cd: not a directory: {}", target_dir.display());
            return Ok(1);
        }

        std::env::set_var("OLDPWD", current_dir.to_string_lossy().to_string());
        *current_dir = target_dir;
        std::env::set_current_dir(&current_dir)?;
        std::env::set_var("PWD", current_dir.to_string_lossy().to_string());

        Ok(0)
    }

    fn name(&self) -> &'static str {
        "cd"
    }

    fn description(&self) -> &'static str {
        "Change the current directory"
    }

    fn usage(&self) -> &'static str {
        "cd [directory]\n  directory  The directory to change to (default: home directory)\n  -          Change to previous directory"
    }
}