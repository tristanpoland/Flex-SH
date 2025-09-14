use crate::cli::Cli;
use crate::config::Config;
use crate::terminal::Terminal;
use crate::core::{executor::Executor, history::History, parser::Parser};
use anyhow::Result;
use log::{debug, info, warn};
use rustyline::DefaultEditor;
use std::path::PathBuf;

pub struct Shell {
    config: Config,
    terminal: Terminal,
    editor: DefaultEditor,
    history: History,
    parser: Parser,
    executor: Executor,
    current_dir: PathBuf,
    exit_code: i32,
    should_exit: bool,
}

impl Shell {
    pub async fn new(args: Cli) -> Result<Self> {
        let config = Config::new(args.config)?;
        let terminal = Terminal::new(config.get().colors.enabled && !args.no_color)?;
        let editor = DefaultEditor::new()?;
        let history = History::new(config.get().history.clone())?;
        let parser = Parser::new();
        let executor = Executor::new();
        let current_dir = std::env::current_dir()?;

        info!("Shell initialized with config: {:?}", config.get());

        Ok(Self {
            config,
            terminal,
            editor,
            history,
            parser,
            executor,
            current_dir,
            exit_code: 0,
            should_exit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting shell main loop");

        if let Err(e) = self.terminal.enter_raw_mode().await {
            warn!("Failed to enter raw mode: {}", e);
        }

        loop {
            if self.should_exit {
                break;
            }

            match self.run_interactive().await {
                Ok(_) => {}
                Err(e) => {
                    self.terminal.print_error(&format!("Shell error: {}", e)).await?;
                    self.exit_code = 1;
                }
            }
        }

        self.terminal.leave_raw_mode().await?;
        info!("Shell exiting with code: {}", self.exit_code);

        Ok(())
    }

    async fn run_interactive(&mut self) -> Result<()> {
        let prompt = self.build_prompt()?;

        match self.editor.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    return Ok(());
                }

                self.history.add(&line.to_string())?;
                self.editor.add_history_entry(line)?;

                debug!("Processing command: {}", line);

                let parsed_command = self.parser.parse(line)?;
                debug!("Parsed command: {:?}", parsed_command);

                self.exit_code = self.executor.execute(parsed_command, &mut self.current_dir).await?;

                if self.exit_code == 130 {
                    self.should_exit = true;
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                self.terminal.print_info("^C").await?;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                self.terminal.print_info("exit").await?;
                self.should_exit = true;
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        Ok(())
    }

    fn build_prompt(&self) -> Result<String> {
        let config = self.config.get();
        let mut prompt = config.prompt.format.clone();

        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "user".to_string());

        let host = gethostname::gethostname()
            .to_string_lossy()
            .to_string();

        let cwd = self.current_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());

        prompt = prompt.replace("{user}", &user);
        prompt = prompt.replace("{host}", &host);
        prompt = prompt.replace("{cwd}", &cwd);

        if config.prompt.show_exit_code && self.exit_code != 0 {
            prompt = format!("[{}] {}", self.exit_code, prompt);
        }

        if config.prompt.show_time {
            let time = chrono::Local::now().format("%H:%M:%S");
            prompt = format!("[{}] {}", time, prompt);
        }

        if config.colors.enabled {
            prompt = self.terminal.colorize_prompt(&prompt);
        }

        Ok(prompt)
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}