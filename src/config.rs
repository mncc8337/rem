use crate::{
    entry::Entry,
    error::RemError,
};
use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};

pub struct ConfigManager {
    pub config: Config,
    pub config_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub entries: Vec<Entry>,
}

impl ConfigManager {
    pub fn open(config_path: PathBuf) -> Result<Self, RemError> {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| {
                eprintln!("error while reading config: {}", e);
                RemError::FileError
            })?;

        let config: Config = serde_json::from_str(&content)
            .map_err(|e| {
                eprintln!("error while parsing config: {}", e);
                RemError::ParsingError
            })?;

        Ok(Self {
            config,
            config_path,
        })
    }
    
    pub fn save(&self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self.config)?;
        let _ = fs::write(&self.config_path, json);
        Ok(())
    }

    pub fn add_entry(&mut self, name: String, interval: u64, message: String, timeout: i32, urgency: u8, icon: String) {
        let new_ent = Entry {
            name,
            interval,
            message,
            icon,
            timeout,
            urgency,
        };
        self.config.entries.push(new_ent);
        let _ = self.save();
    }

    pub fn remove_entry(&mut self, id: u32) {
        self.config.entries.remove(id as usize);
        let _ = self.save();
    }
}

pub fn generate_config(path: &PathBuf) -> std::io::Result<()> {
    if !path.exists() {
        let ex_config = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/access/example_config.json"));
        let _ = fs::write(path, ex_config);
    }

    Ok(())
}
