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
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
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
        let processed = self.process_color_codes(prompt.to_string());
        log::debug!("Original prompt: {}", prompt);
        log::debug!("Processed prompt: {}", processed);
        self.colored_prompt = processed;
    }

    fn process_color_codes(&self, prompt: String) -> String {
        // Apply colors to the text that comes after each color code
        let mut result = String::new();
        let mut chars: std::iter::Peekable<std::str::Chars> = prompt.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Collect the color code
                let mut color_code = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume the '}'
                        break;
                    }
                    color_code.push(chars.next().unwrap());
                }

                // Apply the color based on the code
                match color_code.as_str() {
                    "red" => result.push_str("\x1b[31m"),
                    "green" => result.push_str("\x1b[32m"),
                    "blue" => result.push_str("\x1b[34m"),
                    "yellow" => result.push_str("\x1b[33m"),
                    "magenta" => result.push_str("\x1b[35m"),
                    "cyan" => result.push_str("\x1b[36m"),
                    "white" => result.push_str("\x1b[37m"),
                    "black" => result.push_str("\x1b[30m"),
                    "bright_red" => result.push_str("\x1b[91m"),
                    "bright_green" => result.push_str("\x1b[92m"),
                    "bright_blue" => result.push_str("\x1b[94m"),
                    "bright_yellow" => result.push_str("\x1b[93m"),
                    "bright_magenta" => result.push_str("\x1b[95m"),
                    "bright_cyan" => result.push_str("\x1b[96m"),
                    "bright_white" => result.push_str("\x1b[97m"),
                    "bright_black" => result.push_str("\x1b[90m"),
                    "reset" => result.push_str("\x1b[0m"),
                    "bold" => result.push_str("\x1b[1m"),
                    "dim" => result.push_str("\x1b[2m"),
                    "italic" => result.push_str("\x1b[3m"),
                    "underline" => result.push_str("\x1b[4m"),
                    _ => {
                        // Unknown color code, put it back as-is
                        result.push('{');
                        result.push_str(&color_code);
                        result.push('}');
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
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

    // Complete executable programs from PATH
    fn complete_programs(&self, prefix: &str) -> Option<Vec<Pair>> {
        debug!("Completing programs for prefix: '{}'", prefix);

        if prefix.is_empty() {
            return None; // Don't complete all programs with empty prefix
        }

        let mut matches = Vec::new();
        let paths = std::env::var("PATH").unwrap_or_default();

        #[cfg(windows)]
        let path_separator = ";";
        #[cfg(not(windows))]
        let path_separator = ":";

        #[cfg(windows)]
        let executable_extensions = vec!["exe", "bat", "cmd", "com"];

        for path_dir in paths.split(path_separator) {
            if path_dir.is_empty() {
                continue;
            }

            let dir_path = Path::new(path_dir);
            if !dir_path.exists() || !dir_path.is_dir() {
                continue;
            }

            if let Ok(entries) = dir_path.read_dir() {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name();
                        let name = file_name.to_string_lossy();

                        // Check if it's an executable
                        #[cfg(windows)]
                        let is_executable = {
                            if let Some(ext) = entry.path().extension() {
                                let ext = ext.to_string_lossy().to_lowercase();
                                executable_extensions.contains(&ext.as_str())
                            } else {
                                false
                            }
                        };

                        #[cfg(not(windows))]
                        let is_executable = {
                            entry.path().is_file() &&
                            entry.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
                        };

                        if is_executable && name.starts_with(prefix) {
                            // Remove file extension on Windows for cleaner completion
                            #[cfg(windows)]
                            let display_name = {
                                if let Some(stem) = entry.path().file_stem() {
                                    stem.to_string_lossy().to_string()
                                } else {
                                    name.to_string()
                                }
                            };

                            #[cfg(not(windows))]
                            let display_name = name.to_string();

                            // Avoid duplicates
                            if !matches.iter().any(|m: &Pair| m.display == display_name) {
                                matches.push(Pair {
                                    display: display_name.clone(),
                                    replacement: display_name,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort matches and limit to reasonable number
        matches.sort_by(|a, b| a.display.cmp(&b.display));
        matches.truncate(100); // Limit to 100 programs to avoid overwhelming user

        debug!("Found {} program matches", matches.len());
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

        // Handle command completion (only at start of line)
        if start == 0 {
            let mut command_matches = Vec::new();

            // Built-in commands first
            let builtin_commands = [
                "cd", "echo", "exit", "help", "history", "ls", "pwd",
                "alias", "env", "which", "clear"
            ];

            for cmd in &builtin_commands {
                if cmd.starts_with(word) {
                    command_matches.push(Pair {
                        display: cmd.to_string(),
                        replacement: cmd.to_string(),
                    });
                }
            }

            // Then add executable programs from PATH
            if let Some(path_matches) = self.complete_programs(&word) {
                command_matches.extend(path_matches);
            }

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
        if default || self.colored_prompt.is_empty() {
            Cow::Borrowed(prompt)
        } else {
            // Return the colored version which has same display width as clean prompt
            Cow::Owned(self.colored_prompt.clone())
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

        // Store the original prompt for highlighting
        if let Some(helper) = self.editor.helper_mut() {
            helper.set_colored_prompt(&prompt);
        }

        // Create clean prompt without color codes for width calculation
        let clean_prompt = self.remove_color_codes(&prompt);

        match self.editor.readline(&clean_prompt) {
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

        // Get user name
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "user".to_string());

        // Get hostname
        let hostname = gethostname::gethostname()
            .to_string_lossy()
            .to_string();

        // Get current working directory (home-relative)
        let cwd_home = if let Some(home) = dirs::home_dir() {
            if self.current_dir.starts_with(&home) {
                let relative = self.current_dir.strip_prefix(&home).unwrap_or(&self.current_dir);
                if relative == Path::new("") {
                    "~".to_string()
                } else {
                    format!("~/{}", relative.to_string_lossy())
                }
            } else {
                self.current_dir.to_string_lossy().to_string()
            }
        } else {
            self.current_dir.to_string_lossy().to_string()
        };

        // Get just directory name
        let cwd_name = self.current_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());

        // Get current time
        let time = chrono::Local::now().format("%H:%M:%S").to_string();

        // Replace all variables (no colors here - handled by color codes)
        prompt = prompt.replace("{user}", &user);
        prompt = prompt.replace("{host}", &hostname);
        prompt = prompt.replace("{hostname}", &hostname);
        prompt = prompt.replace("{cwd}", &cwd_home);
        prompt = prompt.replace("{cwd_name}", &cwd_name);
        prompt = prompt.replace("{time}", &time);

        // Don't process color codes here - let rustyline Highlighter handle it

        // Add exit code if enabled and non-zero
        if config.prompt.show_exit_code && self.exit_code != 0 {
            prompt = format!("[{}] {}", self.exit_code, prompt);
        }

        // Add time if enabled (but not if already in prompt format string)
        if config.prompt.show_time && !config.prompt.format.contains("{time}") {
            prompt = format!("[{}] {}", time, prompt);
        }

        // Don't color the prompt here - rustyline helper will handle coloring

        Ok(prompt)
    }

    fn process_color_codes(&self, mut prompt: String) -> String {
        // Apply colors to the text that comes after each color code
        let mut result = String::new();
        let mut chars: std::iter::Peekable<std::str::Chars> = prompt.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Collect the color code
                let mut color_code = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume the '}'
                        break;
                    }
                    color_code.push(chars.next().unwrap());
                }

                // Apply the color based on the code
                match color_code.as_str() {
                    "red" => result.push_str("\x1b[31m"),
                    "green" => result.push_str("\x1b[32m"),
                    "blue" => result.push_str("\x1b[34m"),
                    "yellow" => result.push_str("\x1b[33m"),
                    "magenta" => result.push_str("\x1b[35m"),
                    "cyan" => result.push_str("\x1b[36m"),
                    "white" => result.push_str("\x1b[37m"),
                    "black" => result.push_str("\x1b[30m"),
                    "bright_red" => result.push_str("\x1b[91m"),
                    "bright_green" => result.push_str("\x1b[92m"),
                    "bright_blue" => result.push_str("\x1b[94m"),
                    "bright_yellow" => result.push_str("\x1b[93m"),
                    "bright_magenta" => result.push_str("\x1b[95m"),
                    "bright_cyan" => result.push_str("\x1b[96m"),
                    "bright_white" => result.push_str("\x1b[97m"),
                    "bright_black" => result.push_str("\x1b[90m"),
                    "reset" => result.push_str("\x1b[0m"),
                    "bold" => result.push_str("\x1b[1m"),
                    "dim" => result.push_str("\x1b[2m"),
                    "italic" => result.push_str("\x1b[3m"),
                    "underline" => result.push_str("\x1b[4m"),
                    _ => {
                        // Unknown color code, put it back as-is
                        result.push('{');
                        result.push_str(&color_code);
                        result.push('}');
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn remove_color_codes(&self, prompt: &str) -> String {
        // Remove color codes from prompt for width calculation
        let mut result = String::new();
        let mut chars: std::iter::Peekable<std::str::Chars> = prompt.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Skip the color code
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    if next_ch == '}' {
                        break;
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
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