use super::ShellConfig;
use anyhow::Result;
use std::path::PathBuf;

pub struct Config {
    config: ShellConfig,
    config_path: Option<PathBuf>,
}

impl Config {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let config = ShellConfig::load(path.clone())?;
        Ok(Self {
            config,
            config_path: path,
        })
    }

    pub fn get(&self) -> &ShellConfig {
        &self.config
    }

    pub fn get_mut(&mut self) -> &mut ShellConfig {
        &mut self.config
    }

    pub fn save(&self) -> Result<()> {
        if let Some(ref path) = self.config_path {
            self.config.save(path)?;
        } else {
            let default_path = dirs::config_dir()
                .map(|d| d.join("flex-sh").join("config.toml"))
                .unwrap_or_else(|| PathBuf::from(".flexsh.toml"));
            self.config.save(&default_path)?;
        }
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.config = ShellConfig::load(self.config_path.clone())?;
        Ok(())
    }
}