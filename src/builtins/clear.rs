use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use crossterm::{execute, terminal::{Clear, ClearType}};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use tokio::process::Child;

pub struct ClearCommand;

#[async_trait::async_trait]
impl BuiltinCommand for ClearCommand {
    async fn execute(
        &self,
        _command: &ParsedCommand,
        _current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
    ) -> Result<i32> {
        execute!(io::stdout(), Clear(ClearType::All))?;
        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        Ok(0)
    }

    fn name(&self) -> &'static str {
        "clear"
    }

    fn description(&self) -> &'static str {
        "Clear the terminal screen"
    }

    fn usage(&self) -> &'static str {
        "clear"
    }
}