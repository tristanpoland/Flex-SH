use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCommand {
    pub program: String,
    pub args: Vec<String>,
    pub input_redirect: Option<String>,
    pub output_redirect: Option<String>,
    pub append_redirect: Option<String>,
    pub background: bool,
    pub pipes: Vec<ParsedCommand>,
    pub environment: HashMap<String, String>,
}

impl ParsedCommand {
    pub fn new(program: String) -> Self {
        Self {
            program,
            args: Vec::new(),
            input_redirect: None,
            output_redirect: None,
            append_redirect: None,
            background: false,
            pipes: Vec::new(),
            environment: HashMap::new(),
        }
    }
}

pub struct Parser {
    aliases: HashMap<String, String>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }

    pub fn parse(&self, input: &str) -> Result<ParsedCommand> {
        let input = input.trim();

        if input.is_empty() {
            return Err(anyhow!("Empty command"));
        }

        let tokens = self.tokenize(input)?;
        let mut tokens = tokens;
        // Alias substitution: if first token is an alias, replace it
        if !tokens.is_empty() {
            if let Some(alias) = self.aliases.get(&tokens[0]) {
                // Split alias value into tokens and replace the first token
                let alias_tokens = self.tokenize(alias)?;
                tokens.splice(0..1, alias_tokens);
            }
        }
        self.parse_tokens(tokens)
    }

    fn tokenize(&self, input: &str) -> Result<Vec<String>> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';
        let mut escape_next = false;

        for ch in input.chars() {
            if escape_next {
                current_token.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_quotes => {
                    escape_next = true;
                }
                '"' | '\'' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quotes = false;
                    } else {
                        current_token.push(ch);
                    }
                }
                ' ' | '\t' if !in_quotes => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if in_quotes {
            return Err(anyhow!("Unterminated quote"));
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        Ok(tokens)
    }

    fn parse_tokens(&self, mut tokens: Vec<String>) -> Result<ParsedCommand> {
        if tokens.is_empty() {
            return Err(anyhow!("No tokens to parse"));
        }

        let mut environment = HashMap::new();
        let mut i = 0;

        while i < tokens.len() {
            if let Some(eq_pos) = tokens[i].find('=') {
                if eq_pos > 0 && tokens[i].chars().nth(0).unwrap().is_alphabetic() {
                    let (var, value) = tokens[i].split_at(eq_pos);
                    environment.insert(var.to_string(), value[1..].to_string());
                    tokens.remove(i);
                    continue;
                }
            }
            i += 1;
        }

        if tokens.is_empty() {
            return Err(anyhow!("No command found after environment variables"));
        }

        let program = if let Some(alias) = self.aliases.get(&tokens[0]) {
            alias.clone()
        } else {
            tokens[0].clone()
        };

        let mut command = ParsedCommand::new(program);
        command.environment = environment;

        let mut i = 1;
        while i < tokens.len() {
            match tokens[i].as_str() {
                "<" => {
                    if i + 1 < tokens.len() {
                        command.input_redirect = Some(tokens[i + 1].clone());
                        i += 2;
                    } else {
                        return Err(anyhow!("Expected filename after '<'"));
                    }
                }
                ">" => {
                    if i + 1 < tokens.len() {
                        command.output_redirect = Some(tokens[i + 1].clone());
                        i += 2;
                    } else {
                        return Err(anyhow!("Expected filename after '>'"));
                    }
                }
                ">>" => {
                    if i + 1 < tokens.len() {
                        command.append_redirect = Some(tokens[i + 1].clone());
                        i += 2;
                    } else {
                        return Err(anyhow!("Expected filename after '>>'"));
                    }
                }
                "&" => {
                    command.background = true;
                    i += 1;
                }
                "|" => {
                    let remaining_tokens: Vec<String> = tokens[i + 1..].to_vec();
                    if !remaining_tokens.is_empty() {
                        let pipe_command = self.parse_tokens(remaining_tokens)?;
                        command.pipes.push(pipe_command);
                    }
                    break;
                }
                _ => {
                    command.args.push(tokens[i].clone());
                    i += 1;
                }
            }
        }

        Ok(command)
    }

    pub fn set_alias(&mut self, name: String, value: String) {
        self.aliases.insert(name, value);
    }

    pub fn remove_alias(&mut self, name: &str) {
        self.aliases.remove(name);
    }

    pub fn list_aliases(&self) -> &HashMap<String, String> {
        &self.aliases
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let parser = Parser::new();
        let result = parser.parse("ls -la").unwrap();
        assert_eq!(result.program, "ls");
        assert_eq!(result.args, vec!["-la"]);
    }

    #[test]
    fn test_quoted_arguments() {
        let parser = Parser::new();
        let result = parser.parse(r#"echo "hello world""#).unwrap();
        assert_eq!(result.program, "echo");
        assert_eq!(result.args, vec!["hello world"]);
    }

    #[test]
    fn test_redirection() {
        let parser = Parser::new();
        let result = parser.parse("cat < input.txt > output.txt").unwrap();
        assert_eq!(result.program, "cat");
        assert_eq!(result.input_redirect, Some("input.txt".to_string()));
        assert_eq!(result.output_redirect, Some("output.txt".to_string()));
    }

    #[test]
    fn test_background() {
        let parser = Parser::new();
        let result = parser.parse("sleep 10 &").unwrap();
        assert_eq!(result.program, "sleep");
        assert_eq!(result.args, vec!["10"]);
        assert!(result.background);
    }

    #[test]
    fn test_environment_variables() {
        let parser = Parser::new();
        let result = parser.parse("VAR=value echo $VAR").unwrap();
        assert_eq!(result.program, "echo");
        assert_eq!(result.environment.get("VAR"), Some(&"value".to_string()));
    }
}