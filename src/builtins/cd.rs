use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use crate::utils::path::expand_tilde;
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
        _parser: &mut crate::core::parser::Parser,
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
                // Expand tilde and normalize the path
                let expanded_path = expand_tilde(path);
                let mut target = if expanded_path.is_absolute() {
                    expanded_path
                } else {
                    let mut t = current_dir.clone();
                    t.push(&expanded_path);
                    t
                };
                target
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

        // Note: We don't test read_dir here because some directories allow navigation
        // but not listing. The actual directory change operation will tell us if access is denied.

        std::env::set_var("OLDPWD", current_dir.to_string_lossy().to_string());

        // Canonicalize the target directory to resolve ./ and ../ properly
        let canonical_dir = if let Ok(canonical) = target_dir.canonicalize() {
            canonical
        } else {
            target_dir
        };

        // Try to actually change directory before updating shell state
        if let Err(e) = std::env::set_current_dir(&canonical_dir) {
            match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    eprintln!("cd: permission denied: {}", canonical_dir.display());
                }
                _ => {
                    eprintln!("cd: {}: {}", canonical_dir.display(), e);
                }
            }
            return Ok(1);
        }

        // Update shell state only if directory change succeeded
        *current_dir = canonical_dir.clone();
        std::env::set_var("PWD", canonical_dir.to_string_lossy().to_string());

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