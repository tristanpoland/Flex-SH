use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub struct AliasCommand;


#[async_trait::async_trait]
impl BuiltinCommand for AliasCommand {
    async fn execute(
        &self,
        command: &ParsedCommand,
        _current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
        parser: &mut crate::core::parser::Parser,
    ) -> Result<i32> {
        if command.args.is_empty() {
            let aliases = parser.list_aliases();
            if aliases.is_empty() {
                println!("alias: no aliases defined");
            } else {
                for (name, value) in aliases {
                    println!("alias {}='{}'", name, value);
                }
            }
        } else {
            for arg in &command.args {
                if let Some(eq_pos) = arg.find('=') {
                    let (name, value) = arg.split_at(eq_pos);
                    let value = &value[1..]; // Skip the '=' character
                    parser.set_alias(name.trim().to_string(), value.trim().to_string());
                    println!("alias {}='{}'", name.trim(), value.trim());
                } else {
                    let aliases = parser.list_aliases();
                    if let Some(val) = aliases.get(arg) {
                        println!("alias {}='{}'", arg, val);
                    } else {
                        println!("alias: {}: not found", arg);
                    }
                }
            }
        }
        Ok(0)
    }

    fn name(&self) -> &'static str {
        "alias"
    }

    fn description(&self) -> &'static str {
        "Create or display command aliases"
    }

    fn usage(&self) -> &'static str {
        "alias [name=value ...] [name ...]\n  name=value  Create or update alias\n  name        Display specific alias"
    }
}