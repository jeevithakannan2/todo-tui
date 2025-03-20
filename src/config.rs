use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub encryption: bool,
}

impl Config {
    fn new() -> Self {
        Self { encryption: false }
    }

    pub fn load() -> Self {
        let loaded: Self = match fs::read_to_string(Self::get_config_path()) {
            Ok(contents) => toml::from_str(&contents).unwrap(),
            Err(_) => Self::new(),
        };
        loaded
    }

    fn get_config_dir() -> PathBuf {
        crate::helpers::get_project_dir().config_dir().to_path_buf()
    }

    fn get_config_path() -> PathBuf {
        Self::get_config_dir().join("config.toml")
    }

    pub fn exists(&self) -> bool {
        Self::get_config_path().exists()
    }

    pub fn save(&self) {
        let data = toml::to_string_pretty(&self).unwrap();
        fs::write(Self::get_config_path(), data).unwrap();
    }
}
