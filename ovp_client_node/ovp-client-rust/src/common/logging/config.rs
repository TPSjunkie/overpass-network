// ./src/common/logging/config.rs

use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::common::logging::logger::Logger;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub log_level: String,
    pub log_file: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            log_level: "info".to_string(),
            log_file: "./log/log.txt".to_string(),
        }
    }

    pub fn load() -> Self {
        let mut config = Self::new();
        let path = Path::new("./config.toml");
        if path.exists() {
            let mut file = fs::File::open(path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let value: Value = toml::from_str(&contents).unwrap();
            config.log_level = value["log_level"].as_str().unwrap().to_string();
            config.log_file = value["log_file"].as_str().unwrap().to_string();
        }
        config
    }

    pub fn save(&self) {
        let path = Path::new("./config.toml");
        let mut file = fs::File::create(path).unwrap();
        let mut contents = String::new();
        contents.push_str(&format!("log_level = \"{}\"\n", self.log_level));
        contents.push_str(&format!("log_file = \"{}\"\n", self.log_file));
        file.write_all(contents.as_bytes()).unwrap();
    }

    pub fn set_log_level(&mut self, log_level: &str) {
        self.log_level = log_level.to_string();
    }

    pub fn set_log_file(&mut self, log_file: &str) {
        self.log_file = log_file.to_string();
    }

    pub fn get_log_level(&self) -> &str {
        &self.log_level
    }

    pub fn get_log_file(&self) -> &str {
        &self.log_file
    }
}

impl Logger for Config {
    fn log(&self, level: &str, message: &str) {
        let mut file = fs::File::create(self.log_file.clone()).unwrap();
        let mut contents = String::new();
        contents.push_str(&format!("{} {}\n", level, message));
        file.write_all(contents.as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load() {
        let config = Config::load();
        assert_eq!(config.log_level, "info");
        assert_eq!(config.log_file, "./log/log.txt");
    }

    #[test]
    fn test_config_save() {
        let mut config = Config::new();
        config.log_level = "debug".to_string();
        config.log_file = "./log/debug.txt".to_string();
        config.save();

        let config = Config::load();
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.log_file, "./log/debug.txt");
    }

    #[test]
    fn test_config_set_log_level() {
        let mut config = Config::new();
        config.set_log_level("debug");
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_config_set_log_file() {
        let mut config = Config::new();
        config.set_log_file("./log/debug.txt");
        assert_eq!(config.log_file, "./log/debug.txt");
    }

    #[test]
    fn test_config_get_log_level() {
        let config = Config::new();
        assert_eq!(config.get_log_level(), "info");
    }

    #[test]
    fn test_config_get_log_file() {
        let config = Config::new();
        assert_eq!(config.get_log_file(), "./log/log.txt");
    }
}
