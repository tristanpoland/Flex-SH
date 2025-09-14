use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub struct WhichCommand;

#[async_trait::async_trait]
impl BuiltinCommand for WhichCommand {
    async fn execute(
        &self,
        command: &ParsedCommand,
        _current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
    ) -> Result<i32> {
        if command.args.is_empty() {
            eprintln!("which: missing operand");
            return Ok(1);
        }

        let mut found_all = true;

        for program in &command.args {
            if let Some(path) = find_in_path(program) {
                println!("{}", path.display());
            } else {
                eprintln!("which: no {} in PATH", program);
                found_all = false;
            }
        }

        Ok(if found_all { 0 } else { 1 })
    }

    fn name(&self) -> &'static str {
        "which"
    }

    fn description(&self) -> &'static str {
        "Locate a command in the PATH"
    }

    fn usage(&self) -> &'static str {
        "which program [program ...]"
    }
}

fn find_in_path(program: &str) -> Option<PathBuf> {
    if let Ok(path_env) = std::env::var("PATH") {
        let path_separator = if cfg!(windows) { ';' } else { ':' };
        let executable_extensions = if cfg!(windows) {
            vec!["", ".exe", ".bat", ".cmd", ".com"]
        } else {
            vec![""]
        };

        for path_dir in path_env.split(path_separator) {
            let path_dir = PathBuf::from(path_dir);

            for ext in &executable_extensions {
                let full_path = path_dir.join(format!("{}{}", program, ext));
                if full_path.is_file() {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(metadata) = full_path.metadata() {
                            if metadata.permissions().mode() & 0o111 == 0 {
                                continue;
                            }
                        }
                    }
                    return Some(full_path);
                }
            }
        }
    }

    None
}