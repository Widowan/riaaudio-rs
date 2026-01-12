use crate::errors::RiaError;
use log::error;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;

pub fn load_config(app_config: &AppConfig) -> Result<ParserConfig, RiaError> {
    let config_file = File::open(&app_config.config_file)?;
    Ok(serde_yaml::from_reader(config_file)?)
}

pub fn load_seen(app_config: &AppConfig) -> HashSet<String> {
    let mut seen = HashSet::new();
    if let Ok(content) = fs::read_to_string(&app_config.seen_file) {
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                seen.insert(trimmed.to_string());
            }
        }
    }
    seen
}

pub fn save_seen(app_config: &AppConfig, seen: &mut HashSet<String>, video_id: &str) -> Result<(), RiaError> {
    seen.insert(video_id.to_string());

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&app_config.seen_file)?;
    writeln!(file, "{}", video_id)?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ParserConfig {
    pub(crate) channels: Vec<String>,
}

#[derive(Debug)]
pub struct AppConfig {
    pub download_dir: String,
    pub config_file: String,
    pub seen_file: String,
    pub auto_update: bool,
}

pub fn upload(file_path: &str) {
    error!("upload: unimplemented");
    let _ = fs::remove_file(file_path);
}