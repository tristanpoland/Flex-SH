use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub struct PwdCommand;

#[async_trait::async_trait]
impl BuiltinCommand for PwdCommand {
    async fn execute(
        &self,
        _command: &ParsedCommand,
        current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
        _parser: &mut crate::core::parser::Parser,
    ) -> Result<i32> {
        println!("{}", current_dir.display());
        Ok(0)
    }

    fn name(&self) -> &'static str {
        "pwd"
    }

    fn description(&self) -> &'static str {
        "Print the current working directory"
    }

    fn usage(&self) -> &'static str {
        "pwd"
    }
}