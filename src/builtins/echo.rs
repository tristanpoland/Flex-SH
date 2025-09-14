use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;

pub struct EchoCommand;

#[async_trait::async_trait]
impl BuiltinCommand for EchoCommand {
    async fn execute(
        &self,
        command: &ParsedCommand,
        _current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
    ) -> Result<i32> {
        let mut newline = true;
        let mut args = command.args.iter();

        if let Some(first_arg) = args.next() {
            if first_arg == "-n" {
                newline = false;
            } else {
                print!("{}", expand_variables(first_arg));
            }
        }

        for arg in args {
            print!(" {}", expand_variables(arg));
        }

        if newline {
            println!();
        }

        Ok(0)
    }

    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Display a line of text"
    }

    fn usage(&self) -> &'static str {
        "echo [-n] [string ...]\n  -n  Do not output trailing newline"
    }
}

fn expand_variables(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            if let Some(&'{') = chars.peek() {
                chars.next(); // consume '{'
                let mut var_name = String::new();
                let mut found_closing = false;

                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        found_closing = true;
                        break;
                    }
                    var_name.push(ch);
                }

                if found_closing {
                    if let Ok(value) = std::env::var(&var_name) {
                        result.push_str(&value);
                    }
                } else {
                    result.push('$');
                    result.push('{');
                    result.push_str(&var_name);
                }
            } else if let Some(&next_ch) = chars.peek() {
                if next_ch.is_alphabetic() || next_ch == '_' {
                    let mut var_name = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            var_name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    if let Ok(value) = std::env::var(&var_name) {
                        result.push_str(&value);
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        } else if ch == '\\' {
            if let Some(escaped) = chars.next() {
                match escaped {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    _ => {
                        result.push('\\');
                        result.push(escaped);
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}