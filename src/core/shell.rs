use crate::cli::Cli;
use crate::config::Config;
use crate::terminal::Terminal;
use crate::core::{executor::Executor, history::History, parser::Parser};
use anyhow::Result;
use colored::*;
use log::{debug, info, warn};
use rustyline::{Editor, Helper, Context, Config as EditorConfig, CompletionType, EditMode};
use rustyline::completion::FilenameCompleter;
use rustyline::history::DefaultHistory;
use rustyline::highlight::Highlighter;
use rustyline_derive::{Completer, Helper, Hinter, Validator};
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Helper, Completer, Hinter, Validator)]
struct ShellHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    colored_prompt: String,
}

impl ShellHelper {
    fn new() -> Self {
        ShellHelper {
            completer: FilenameCompleter::new(),
            colored_prompt: String::new(),
        }
    }

    fn set_colored_prompt(&mut self, prompt: &str) {
        // Create colored version for display
        self.colored_prompt = prompt
            .replace("[", &format!("{}", "[".bright_cyan()))
            .replace("]", &format!("{}", "]".bright_cyan()))
            .replace("$", &format!("{}", "$".bright_magenta().bold()));
    }
}

// Custom Highlighter implementation for prompt coloring
impl Highlighter for ShellHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default && !self.colored_prompt.is_empty() {
            Cow::Borrowed(&self.colored_prompt)
        } else {
            Cow::Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(hint.bright_black().to_string())
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        let _ = pos;
        Cow::Borrowed(line)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        let _ = completion;
        Cow::Borrowed(candidate)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: rustyline::highlight::CmdKind) -> bool {
        let _ = (line, pos, kind);
        false
    }
}

pub struct Shell {
    config: Config,
    terminal: Terminal,
    editor: Editor<ShellHelper, DefaultHistory>,
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

        // Configure the editor with proper settings for completion
        let editor_config = EditorConfig::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();

        let mut editor = Editor::with_config(editor_config)?;
        editor.set_helper(Some(ShellHelper::new()));

        let history = History::new(config.get().history.clone())?;
        let parser = Parser::new();
        let executor = Executor::new();
        let current_dir = std::env::current_dir()?;

        debug!("Shell initialized with config: {:?}", config.get());

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
        debug!("Starting shell main loop");

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

        // Set the colored prompt in the helper
        if let Some(helper) = self.editor.helper_mut() {
            helper.set_colored_prompt(&prompt);
        }

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

        // Don't color the prompt here - rustyline helper will handle coloring

        Ok(prompt)
    }

    pub async fn execute_command(&mut self, command: &str) -> Result<()> {
        debug!("Executing single command: {}", command);

        let parsed_command = self.parser.parse(command)?;
        debug!("Parsed command: {:?}", parsed_command);

        self.exit_code = self.executor.execute(parsed_command, &mut self.current_dir).await?;

        Ok(())
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}