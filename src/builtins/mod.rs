use crate::core::parser::ParsedCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub mod cd;
pub mod echo;
pub mod exit;
pub mod help;
pub mod history;
pub mod ls;
pub mod pwd;
pub mod alias;
pub mod env;
pub mod which;
pub mod clear;

#[async_trait::async_trait]
pub trait BuiltinCommand: Send + Sync {
    async fn execute(
        &self,
        command: &ParsedCommand,
        current_dir: &mut PathBuf,
        background_processes: &mut HashMap<u32, Child>,
        parser: &mut crate::core::parser::Parser,
    ) -> Result<i32>;

    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn usage(&self) -> &'static str;
}

pub fn get_builtin(name: &str) -> Option<Box<dyn BuiltinCommand>> {
    match name {
        "cd" => Some(Box::new(cd::CdCommand)),
        "echo" => Some(Box::new(echo::EchoCommand)),
        "exit" => Some(Box::new(exit::ExitCommand)),
        "help" => Some(Box::new(help::HelpCommand)),
        "history" => Some(Box::new(history::HistoryCommand)),
        "ls" => Some(Box::new(ls::LsCommand)),
        "pwd" => Some(Box::new(pwd::PwdCommand)),
        "alias" => Some(Box::new(alias::AliasCommand)),
        "env" => Some(Box::new(env::EnvCommand)),
        "which" => Some(Box::new(which::WhichCommand)),
        "clear" => Some(Box::new(clear::ClearCommand)),
        _ => None,
    }
}

pub fn list_builtins() -> Vec<&'static str> {
    vec![
        "cd", "echo", "exit", "help", "history", "ls", "pwd", "alias", "env", "which", "clear"
    ]
}