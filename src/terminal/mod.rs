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

        let mut colored_prompt = String::new();
        let mut in_brackets = false;
        let mut current_segment = String::new();

        for ch in prompt.chars() {
            match ch {
                '[' => {
                    if !current_segment.is_empty() {
                        colored_prompt.push_str(&current_segment);
                        current_segment.clear();
                    }
                    in_brackets = true;
                    current_segment.push(ch);
                }
                ']' => {
                    current_segment.push(ch);
                    if in_brackets {
                        colored_prompt.push_str(&current_segment.bright_blue().to_string());
                        current_segment.clear();
                        in_brackets = false;
                    }
                }
                '$' | '%' | '#' => {
                    current_segment.push(ch);
                    if !in_brackets {
                        colored_prompt.push_str(&current_segment.bright_magenta().bold().to_string());
                        current_segment.clear();
                    }
                }
                _ => {
                    current_segment.push(ch);
                }
            }
        }

        if !current_segment.is_empty() {
            if in_brackets {
                colored_prompt.push_str(&current_segment.bright_blue().to_string());
            } else {
                colored_prompt.push_str(&current_segment);
            }
        }

        colored_prompt
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