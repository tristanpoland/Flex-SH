use crate::config::HistoryConfig;
use anyhow::Result;
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct History {
    entries: VecDeque<String>,
    config: HistoryConfig,
    file_path: Option<PathBuf>,
}

impl History {
    pub fn new(config: HistoryConfig) -> Result<Self> {
        let file_path = config.file_path.clone()
            .or_else(|| dirs::data_dir().map(|d| d.join("flex-sh").join("history")));

        let mut history = Self {
            entries: VecDeque::with_capacity(config.max_entries),
            config,
            file_path,
        };

        history.load_from_file()?;
        Ok(history)
    }

    pub fn add(&mut self, command: &String) -> Result<()> {
        if command.trim().is_empty() {
            return Ok(());
        }

        if self.config.ignore_space_prefixed && command.starts_with(' ') {
            return Ok(());
        }

        if self.config.ignore_duplicates && self.entries.back() == Some(command) {
            return Ok(());
        }

        while self.entries.len() >= self.config.max_entries {
            self.entries.pop_front();
        }

        self.entries.push_back(command.clone());
        self.save_to_file()?;

        Ok(())
    }

    pub fn get_entries(&self) -> &VecDeque<String> {
        &self.entries
    }

    pub fn search(&self, pattern: &str) -> Vec<String> {
        self.entries
            .iter()
            .rev()
            .filter(|entry| entry.contains(pattern))
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.save_to_file()?;
        Ok(())
    }

    fn load_from_file(&mut self) -> Result<()> {
        if let Some(ref path) = self.file_path {
            if path.exists() {
                let file = std::fs::File::open(path)?;
                let reader = BufReader::new(file);

                for line in reader.lines() {
                    let line = line?;
                    if !line.trim().is_empty() {
                        self.entries.push_back(line);
                    }
                }

                while self.entries.len() > self.config.max_entries {
                    self.entries.pop_front();
                }
            }
        }
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        if let Some(ref path) = self.file_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?;

            for entry in &self.entries {
                writeln!(file, "{}", entry)?;
            }

            file.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> HistoryConfig {
        HistoryConfig {
            max_entries: 100,
            file_path: None,
            ignore_duplicates: true,
            ignore_space_prefixed: true,
        }
    }

    #[test]
    fn test_add_command() {
        let mut history = History::new(test_config()).unwrap();
        history.add(&"ls -la".to_string()).unwrap();
        assert_eq!(history.entries.len(), 1);
        assert_eq!(history.entries.back().unwrap(), "ls -la");
    }

    #[test]
    fn test_ignore_duplicates() {
        let mut history = History::new(test_config()).unwrap();
        history.add(&"ls".to_string()).unwrap();
        history.add(&"ls".to_string()).unwrap();
        assert_eq!(history.entries.len(), 1);
    }

    #[test]
    fn test_ignore_space_prefixed() {
        let mut history = History::new(test_config()).unwrap();
        history.add(&" secret command".to_string()).unwrap();
        assert_eq!(history.entries.len(), 0);
    }

    #[test]
    fn test_max_entries() {
        let config = HistoryConfig {
            max_entries: 2,
            file_path: None,
            ignore_duplicates: false,
            ignore_space_prefixed: false,
        };
        let mut history = History::new(config).unwrap();

        history.add(&"cmd1".to_string()).unwrap();
        history.add(&"cmd2".to_string()).unwrap();
        history.add(&"cmd3".to_string()).unwrap();

        assert_eq!(history.entries.len(), 2);
        assert_eq!(history.entries.front().unwrap(), "cmd2");
        assert_eq!(history.entries.back().unwrap(), "cmd3");
    }

    #[test]
    fn test_search() {
        let mut history = History::new(test_config()).unwrap();
        history.add(&"git status".to_string()).unwrap();
        history.add(&"git commit".to_string()).unwrap();
        history.add(&"ls -la".to_string()).unwrap();

        let results = history.search("git");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"git status".to_string()));
        assert!(results.contains(&"git commit".to_string()));
    }
}