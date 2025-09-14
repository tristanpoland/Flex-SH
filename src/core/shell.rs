use crate::cli::Cli;
use crate::config::Config;
use crate::terminal::Terminal;
use crate::core::{executor::Executor, history::History, parser::Parser};
use anyhow::Result;
use colored::*;
use log::{debug, info, warn};
use rustyline::{Editor, Helper, Context, Config as EditorConfig, CompletionType, EditMode};
use rustyline::completion::{Completer, Pair, extract_word};
use std::path::{Path, PathBuf};
use rustyline::history::DefaultHistory;
use rustyline::highlight::Highlighter;
use rustyline_derive::{Helper, Hinter, Validator};
use std::borrow::Cow;

#[derive(Helper, Hinter, Validator)]
struct ShellHelper {
    colored_prompt: String,
}

impl ShellHelper {
    fn new() -> Self {
        ShellHelper {
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

    // Manual path completion for all path scenarios
    fn complete_complex_path(&self, word: &str, _start: usize) -> Option<Vec<Pair>> {
        debug!("Attempting path completion for: '{}'", word);

        // Handle simple filenames in current directory
        let (dir_part, file_part) = if word.is_empty() {
            (String::new(), String::new())
        } else if word.ends_with('/') || word.ends_with('\\') {
            // Directory listing - remove trailing slash
            let clean_word = word.trim_end_matches('/').trim_end_matches('\\');
            (clean_word.to_string(), String::new())
        } else if word.contains('/') || word.contains('\\') {
            // Path with directory component
            let path = Path::new(word);
            match path.parent() {
                Some(parent) => {
                    let dir = parent.to_string_lossy().to_string();
                    let file = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    (dir, file)
                },
                None => (String::new(), word.to_string()),
            }
        } else {
            // Simple filename in current directory
            (String::new(), word.to_string())
        };

        debug!("Split path: dir='{}', file='{}'", dir_part, file_part);

        // Resolve the directory path
        let current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return None,
        };

        let target_dir = if dir_part.is_empty() {
            current_dir
        } else {
            let dir_path = Path::new(&dir_part);
            if dir_path.is_absolute() {
                dir_path.to_path_buf()
            } else {
                current_dir.join(dir_path)
            }
        };

        debug!("Target directory: {:?}", target_dir);

        // Try to canonicalize (resolve ./ and ../ components)
        let resolved_dir = match target_dir.canonicalize() {
            Ok(dir) => dir,
            Err(e) => {
                debug!("Failed to canonicalize directory: {:?}", e);
                return None;
            }
        };

        debug!("Resolved directory: {:?}", resolved_dir);

        // Read directory contents
        let entries = match resolved_dir.read_dir() {
            Ok(entries) => entries,
            Err(e) => {
                debug!("Failed to read directory: {:?}", e);
                return None;
            }
        };

        let mut matches = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();

                // Filter based on file_part prefix
                if name.starts_with(&file_part) {
                    let display = name.to_string();

                    // Determine the proper replacement based on context
                    let replacement = if file_part.is_empty() {
                        // Directory listing case - word ends with slash or is a complete directory name
                        let base = if word.ends_with('/') || word.ends_with('\\') {
                            // Replace from the slash onwards
                            if dir_part.is_empty() {
                                format!("./{}", name)
                            } else {
                                format!("{}/{}", dir_part, name)
                            }
                        } else {
                            // Complete directory name
                            if dir_part.is_empty() {
                                name.to_string()
                            } else {
                                format!("{}/{}", dir_part, name)
                            }
                        };

                        // Add trailing slash for directories
                        if entry.path().is_dir() {
                            format!("{}/", base)
                        } else {
                            base
                        }
                    } else {
                        // Partial filename completion
                        let base = if dir_part.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}/{}", dir_part, name)
                        };

                        // Add trailing slash for directories
                        if entry.path().is_dir() {
                            format!("{}/", base)
                        } else {
                            base
                        }
                    };

                    matches.push(Pair {
                        display,
                        replacement,
                    });
                }
            }
        }

        debug!("Found {} manual matches", matches.len());
        if matches.is_empty() { None } else { Some(matches) }
    }
}

// Custom Completer implementation for better relative path handling
impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // Extract the word at cursor position with custom break characters
        // Include . and / as part of the word for relative paths
        let (start, word) = extract_word(line, pos, None, |c| {
            c == ' ' || c == '\t' || c == '\n' || c == '\r'
        });

        debug!("Completion: line='{}', pos={}, start={}, word='{}'", line, pos, start, word);

        debug!("Completion request for word: '{}' at position {} (start={})", word, pos, start);

        // Handle built-in commands (only at start of line)
        if start == 0 {
            let builtin_commands = [
                "cd", "echo", "exit", "help", "history", "ls", "pwd",
                "alias", "env", "which", "clear"
            ];

            let command_matches: Vec<Pair> = builtin_commands
                .iter()
                .filter(|cmd| cmd.starts_with(word))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();

            if !command_matches.is_empty() {
                debug!("Found {} command matches", command_matches.len());
                return Ok((start, command_matches));
            }
        }

        // Handle ALL other completion (files, paths, arguments) manually
        debug!("Attempting file/path completion");
        if let Some(file_matches) = self.complete_complex_path(&word, start) {
            debug!("Found {} file/path matches", file_matches.len());
            return Ok((start, file_matches));
        }

        // No matches found
        debug!("No completion matches found");
        Ok((start, Vec::new()))
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