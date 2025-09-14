use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "flex-sh",
    version,
    about = "A high-performance, modern system shell",
    long_about = "Flex-SH is a cross-platform system shell with rich features including syntax highlighting, auto-completion, and advanced command parsing."
)]
pub struct Cli {
    /// Configuration file path
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Execute a single command and exit
    #[arg(short = 'c', long)]
    pub command: Option<String>,

    /// Interactive mode (default)
    #[arg(short, long)]
    pub interactive: bool,

    /// Verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Disable colors
    #[arg(long)]
    pub no_color: bool,

    /// Script file to execute
    pub script: Option<PathBuf>,
}