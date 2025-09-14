pub mod colors;
pub mod events;
pub mod interface;

use anyhow::Result;
use colored::*;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

pub struct Terminal {
    colors_enabled: bool,
}

impl Terminal {
    pub fn new(colors_enabled: bool) -> Result<Self> {
        Ok(Self { colors_enabled })
    }

    pub async fn enter_raw_mode(&self) -> Result<()> {
        enable_raw_mode()?;
        Ok(())
    }

    pub async fn leave_raw_mode(&self) -> Result<()> {
        disable_raw_mode()?;
        Ok(())
    }

    pub async fn print_info(&self, message: &str) -> Result<()> {
        if self.colors_enabled {
            println!("{}", message.bright_blue());
        } else {
            println!("{}", message);
        }
        Ok(())
    }

    pub async fn print_error(&self, message: &str) -> Result<()> {
        if self.colors_enabled {
            eprintln!("{}", message.bright_red());
        } else {
            eprintln!("{}", message);
        }
        Ok(())
    }

    pub async fn print_success(&self, message: &str) -> Result<()> {
        if self.colors_enabled {
            println!("{}", message.bright_green());
        } else {
            println!("{}", message);
        }
        Ok(())
    }

    pub async fn print_warning(&self, message: &str) -> Result<()> {
        if self.colors_enabled {
            println!("{}", message.bright_yellow());
        } else {
            println!("{}", message);
        }
        Ok(())
    }

    pub fn colorize_prompt(&self, prompt: &str) -> String {
        if !self.colors_enabled {
            return prompt.to_string();
        }

        // Apply colors with proper rustyline invisible character markers
        let mut result = String::new();
        let mut chars = prompt.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '[' => {
                    // Collect bracket content
                    let mut bracket_content = String::new();
                    bracket_content.push('[');

                    while let Some(inner_ch) = chars.next() {
                        bracket_content.push(inner_ch);
                        if inner_ch == ']' {
                            break;
                        }
                    }

                    // Get the ANSI-colored version
                    let colored_text = bracket_content.bright_cyan().to_string();

                    // Wrap ONLY the ANSI codes in \x01..\x02, not the visible text
                    // This is tricky - we need to separate ANSI codes from visible chars
                    let visible_chars = &bracket_content;
                    result.push_str("\x01");
                    result.push_str(&format!("\x1b[96m")); // bright cyan
                    result.push_str("\x02");
                    result.push_str(visible_chars);
                    result.push_str("\x01");
                    result.push_str("\x1b[0m"); // reset
                    result.push_str("\x02");
                }
                '$' | '#' | '%' => {
                    // Color prompt symbols
                    result.push_str("\x01");
                    result.push_str(&format!("\x1b[95;1m")); // bright magenta bold
                    result.push_str("\x02");
                    result.push(ch);
                    result.push_str("\x01");
                    result.push_str("\x1b[0m"); // reset
                    result.push_str("\x02");
                }
                ' ' => {
                    // Preserve spaces exactly
                    result.push(' ');
                }
                _ => {
                    // Regular characters
                    result.push(ch);
                }
            }
        }

        result
    }

    pub fn colorize_output(&self, text: &str, color_type: OutputColorType) -> String {
        if !self.colors_enabled {
            return text.to_string();
        }

        match color_type {
            OutputColorType::Command => text.bright_blue().to_string(),
            OutputColorType::Argument => text.white().to_string(),
            OutputColorType::Error => text.bright_red().to_string(),
            OutputColorType::Success => text.bright_green().to_string(),
            OutputColorType::Info => text.bright_cyan().to_string(),
            OutputColorType::Warning => text.bright_yellow().to_string(),
            OutputColorType::Path => text.bright_cyan().underline().to_string(),
            OutputColorType::Number => text.bright_magenta().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputColorType {
    Command,
    Argument,
    Error,
    Success,
    Info,
    Warning,
    Path,
    Number,
}