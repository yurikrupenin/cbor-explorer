use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use directories::ProjectDirs;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub show_hex_integers: bool,
}

/// Default theme on WASM xterm is incredibly ugly, and it's not like
/// it is an actual user preference. Choose something pretty instead.
impl Default for AppConfig {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let theme = "Rosé Pine Moon".to_string();
        #[cfg(not(target_arch = "wasm32"))]
        let theme = "Default".to_string();

        Self {
            theme,
            show_hex_integers: false,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct ConfigStore {
    config_path: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
pub struct ConfigStore;

#[cfg(target_arch = "wasm32")]
impl ConfigStore {
    pub fn new() -> Option<Self> {
        Some(Self)
    }

    pub fn load(&self) -> AppConfig {
        AppConfig::default()
    }

    pub fn save(&self, _config: &AppConfig) {
        // TODO: save config to local storage/cookies?..
    }
}
