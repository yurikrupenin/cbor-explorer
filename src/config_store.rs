use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub show_hex_integers: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "Tokyo Night".to_string(), // Default theme
            show_hex_integers: false,
        }
    }
}

pub struct ConfigStore {
    config_path: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Option<Self> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "CborExplorer", "cbx") {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = fs::create_dir_all(config_dir);
            }
            Some(Self {
                config_path: config_dir.join("config.toml"),
            })
        } else {
            None
        }
    }

    pub fn load(&self) -> AppConfig {
        if self.config_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        AppConfig::default()
    }

    pub fn save(&self, config: &AppConfig) {
        if let Ok(content) = toml::to_string_pretty(config) {
            let _ = fs::write(&self.config_path, content);
        }
    }
}
