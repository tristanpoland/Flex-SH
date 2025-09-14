use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use log::{error, info, debug};

mod cli;
mod config;
mod core;
mod builtins;
mod terminal;
mod utils;

use cli::Cli;
use core::Shell;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Cli::parse();

    debug!("Starting Flex-SH v{}", env!("CARGO_PKG_VERSION"));

    let mut shell = Shell::new(args.clone()).await?;

    if let Some(command) = args.command {
        // Execute single command and exit
        if let Err(e) = shell.execute_command(&command).await {
            error!("Command error: {}", e);
            std::process::exit(1);
        }
    } else {
        // Enter interactive mode
        if let Err(e) = shell.run().await {
            error!("Shell error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}