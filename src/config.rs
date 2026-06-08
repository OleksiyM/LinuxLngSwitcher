use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppConfig {
    pub left_ctrl_layout: u32,
    pub right_ctrl_layouts: Vec<u32>,
    pub sensitivity_ms: u64,
    #[serde(default)]
    pub window_width: Option<i32>,
    #[serde(default)]
    pub window_height: Option<i32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            left_ctrl_layout: 0,
            right_ctrl_layouts: vec![1],
            sensitivity_ms: 300,
            window_width: None,
            window_height: None,
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config").join("gnome-lng-switcher")
}

pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config.json")
}

pub fn get_pid_path() -> PathBuf {
    get_config_dir().join("daemon.pid")
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if !path.exists() {
        return AppConfig::default();
    }

    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return AppConfig::default(),
    };

    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        return AppConfig::default();
    }

    serde_json::from_str(&content).unwrap_or_else(|_| AppConfig::default())
}

pub fn save_config(config: &AppConfig) -> Result<(), std::io::Error> {
    let dir = get_config_dir();
    create_dir_all(&dir)?;

    let path = get_config_path();
    let mut file = File::create(&path)?;
    let content = serde_json::to_string_pretty(config)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
