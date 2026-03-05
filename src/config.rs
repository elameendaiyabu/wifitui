use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
}

#[derive(Deserialize, Debug)]
pub struct GeneralConfig {
    #[serde(default = "default_tick_rate")]
    pub tick_rate: u64,

    #[serde(default = "default_auto_scan")]
    pub auto_scan_interval: u64,

    #[serde(default = "default_auto_refresh")]
    pub auto_refresh_interval: u64,
}

fn default_tick_rate() -> u64 {
    250
}

fn default_auto_scan() -> u64 {
    30
}

fn default_auto_refresh() -> u64 {
    10
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            tick_rate: default_tick_rate(),
            auto_scan_interval: default_auto_scan(),
            auto_refresh_interval: default_auto_refresh(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn load(config_path: Option<&PathBuf>) -> Self {
        let path = config_path.cloned().unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_default()
                .join("wifitui")
                .join("config.toml")
        });

        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Warning: failed to parse config: {e}");
                    }
                },
                Err(e) => {
                    eprintln!("Warning: failed to read config: {e}");
                }
            }
        }

        Self::default()
    }
}
