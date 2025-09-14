use crate::builtins::list_builtins;
use crate::utils::path::{expand_tilde, get_parent_and_name, is_hidden};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    pub text: String,
    pub display: String,
    pub kind: CompletionKind,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompletionKind {
    Command,
    File,
    Directory,
    Builtin,
    Variable,
    Alias,
}

pub struct CompletionEngine {
    show_hidden: bool,
    case_sensitive: bool,
}

impl CompletionEngine {
    pub fn new(show_hidden: bool, case_sensitive: bool) -> Self {
        Self {
            show_hidden,
            case_sensitive,
        }
    }

    pub fn complete(&self, input: &str, cursor_pos: usize) -> Vec<CompletionCandidate> {
        let (prefix, word_start) = self.extract_word_at_cursor(input, cursor_pos);

        if word_start == 0 {
            // First word - complete commands
            self.complete_commands(&prefix)
        } else {
            // Arguments - complete files/directories
            self.complete_paths(&prefix)
        }
    }

    fn extract_word_at_cursor(&self, input: &str, cursor_pos: usize) -> (String, usize) {
        let chars: Vec<char> = input.chars().collect();
        let cursor_pos = cursor_pos.min(chars.len());

        let mut start = cursor_pos;
        while start > 0 && !chars[start - 1].is_whitespace() {
            start -= 1;
        }

        let mut end = cursor_pos;
        while end < chars.len() && !chars[end].is_whitespace() {
            end += 1;
        }

        let prefix: String = chars[start..cursor_pos].iter().collect();
        (prefix, start)
    }

    fn complete_commands(&self, prefix: &str) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();

        // Add builtin commands
        for builtin in list_builtins() {
            if self.matches_prefix(builtin, prefix) {
                candidates.push(CompletionCandidate {
                    text: builtin.to_string(),
                    display: builtin.to_string(),
                    kind: CompletionKind::Builtin,
                    description: Some("built-in command".to_string()),
                });
            }
        }

        // Add commands from PATH
        if let Ok(path_env) = std::env::var("PATH") {
            let path_separator = if cfg!(windows) { ';' } else { ':' };
            let mut seen = HashSet::new();

            for path_dir in path_env.split(path_separator) {
                if let Ok(entries) = fs::read_dir(path_dir) {
                    for entry in entries.flatten() {
                        if let Ok(file_name) = entry.file_name().into_string() {
                            if seen.contains(&file_name) {
                                continue;
                            }

                            if self.matches_prefix(&file_name, prefix) {
                                if self.is_executable(&entry.path()) {
                                    seen.insert(file_name.clone());
                                    candidates.push(CompletionCandidate {
                                        text: file_name.clone(),
                                        display: file_name,
                                        kind: CompletionKind::Command,
                                        description: Some("executable".to_string()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        candidates.sort_by(|a, b| {
            // Sort builtins first, then by name
            match (a.kind, b.kind) {
                (CompletionKind::Builtin, CompletionKind::Command) => std::cmp::Ordering::Less,
                (CompletionKind::Command, CompletionKind::Builtin) => std::cmp::Ordering::Greater,
                _ => a.text.cmp(&b.text),
            }
        });

        candidates
    }

    fn complete_paths(&self, prefix: &str) -> Vec<CompletionCandidate> {
        let expanded_prefix = expand_tilde(prefix);
        let (parent_dir, file_prefix) = get_parent_and_name(&expanded_prefix);

        let search_dir = if parent_dir == Path::new(".") && prefix.contains('/') {
            // Handle cases like "some/path"
            if let Some(parent) = expanded_prefix.parent() {
                parent.to_path_buf()
            } else {
                parent_dir
            }
        } else {
            parent_dir
        };

        let mut candidates = Vec::new();

        if let Ok(entries) = fs::read_dir(&search_dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if !self.show_hidden && is_hidden(&file_name) {
                        continue;
                    }

                    if self.matches_prefix(&file_name, &file_prefix) {
                        let full_path = search_dir.join(&file_name);
                        let is_dir = full_path.is_dir();

                        let display_name = if is_dir {
                            format!("{}/", file_name)
                        } else {
                            file_name.clone()
                        };

                        candidates.push(CompletionCandidate {
                            text: file_name,
                            display: display_name,
                            kind: if is_dir {
                                CompletionKind::Directory
                            } else {
                                CompletionKind::File
                            },
                            description: None,
                        });
                    }
                }
            }
        }

        candidates.sort_by(|a, b| {
            // Sort directories first, then files
            match (a.kind, b.kind) {
                (CompletionKind::Directory, CompletionKind::File) => std::cmp::Ordering::Less,
                (CompletionKind::File, CompletionKind::Directory) => std::cmp::Ordering::Greater,
                _ => a.text.cmp(&b.text),
            }
        });

        candidates
    }

    fn matches_prefix(&self, candidate: &str, prefix: &str) -> bool {
        if prefix.is_empty() {
            return true;
        }

        if self.case_sensitive {
            candidate.starts_with(prefix)
        } else {
            candidate.to_lowercase().starts_with(&prefix.to_lowercase())
        }
    }

    fn is_executable(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = path.metadata() {
                return metadata.permissions().mode() & 0o111 != 0;
            }
        }

        #[cfg(windows)]
        {
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                return matches!(ext.as_str(), "exe" | "bat" | "cmd" | "com" | "ps1");
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_word_at_cursor() {
        let engine = CompletionEngine::new(false, true);

        let (prefix, start) = engine.extract_word_at_cursor("ls -la", 2);
        assert_eq!(prefix, "ls");
        assert_eq!(start, 0);

        let (prefix, start) = engine.extract_word_at_cursor("ls -la", 4);
        assert_eq!(prefix, "-");
        assert_eq!(start, 3);

        let (prefix, start) = engine.extract_word_at_cursor("echo hello", 6);
        assert_eq!(prefix, "h");
        assert_eq!(start, 5);
    }

    #[test]
    fn test_matches_prefix() {
        let engine = CompletionEngine::new(false, true);
        assert!(engine.matches_prefix("ls", "l"));
        assert!(engine.matches_prefix("ls", "ls"));
        assert!(!engine.matches_prefix("ls", "x"));

        let engine_case_insensitive = CompletionEngine::new(false, false);
        assert!(engine_case_insensitive.matches_prefix("Ls", "l"));
        assert!(engine_case_insensitive.matches_prefix("LS", "ls"));
    }

    #[test]
    fn test_complete_commands() {
        let engine = CompletionEngine::new(false, true);
        let candidates = engine.complete_commands("l");

        // Should contain ls builtin
        assert!(candidates.iter().any(|c| c.text == "ls" && c.kind == CompletionKind::Builtin));
    }
}