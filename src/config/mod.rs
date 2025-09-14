pub mod settings;

pub use settings::Config;

use anyhow::Result;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub prompt: PromptConfig,
    pub colors: ColorConfig,
    pub history: HistoryConfig,
    pub completion: CompletionConfig,
    pub aliases: std::collections::HashMap<String, String>,
    pub environment: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub format: String,
    pub show_git: bool,
    pub show_time: bool,
    pub show_exit_code: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub enabled: bool,
    pub scheme: String,
    pub command_color: String,
    pub argument_color: String,
    pub error_color: String,
    pub success_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub max_entries: usize,
    pub file_path: Option<PathBuf>,
    pub ignore_duplicates: bool,
    pub ignore_space_prefixed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionConfig {
    pub enabled: bool,
    pub case_sensitive: bool,
    pub fuzzy_matching: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            prompt: PromptConfig {
                format: "[{user}@{host} {cwd}]$ ".to_string(),
                show_git: true,
                show_time: false,
                show_exit_code: true,
            },
            colors: ColorConfig {
                enabled: true,
                scheme: "default".to_string(),
                command_color: "bright_blue".to_string(),
                argument_color: "white".to_string(),
                error_color: "bright_red".to_string(),
                success_color: "bright_green".to_string(),
            },
            history: HistoryConfig {
                max_entries: 10000,
                file_path: None,
                ignore_duplicates: true,
                ignore_space_prefixed: true,
            },
            completion: CompletionConfig {
                enabled: true,
                case_sensitive: false,
                fuzzy_matching: true,
            },
            aliases: std::collections::HashMap::new(),
            environment: std::collections::HashMap::new(),
        }
    }
}

impl ShellConfig {
    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        if let Some(config_path) = path {
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let config: ShellConfig = toml::from_str(&content)?;
                return Ok(config);
            }
        }

        let default_config_paths = [
            dirs::config_dir().map(|d| d.join("flex-sh").join("config.toml")),
            dirs::home_dir().map(|d| d.join(".config").join("flex-sh").join("config.toml")),
            Some(PathBuf::from("flex-sh-config.toml")),  // Current directory
            Some(PathBuf::from("config.toml")),
            Some(PathBuf::from(".flexsh.toml")),
        ];

        debug!("Looking for config files in the following locations:");
        for (i, path) in default_config_paths.iter().enumerate() {
            if let Some(path) = path {
                debug!("  {}: {:?}", i + 1, path);
            }
        }

        for config_path in default_config_paths.iter().flatten() {
            debug!("Checking config path: {:?}", config_path);
            if config_path.exists() {
                debug!("Found config at: {:?}", config_path);
                let content = std::fs::read_to_string(config_path)?;
                let config: ShellConfig = toml::from_str(&content)?;
                return Ok(config);
            }
        }

        Ok(ShellConfig::default())
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}