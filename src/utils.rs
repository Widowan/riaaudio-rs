use crate::errors::RiaError;
use reqwest::blocking::multipart::Form as MultipartForm;
use reqwest::blocking::multipart::Part as MultipartPart;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use log::debug;

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
    pub channels: Vec<String>,
}

#[derive(Debug)]
pub struct AppConfig {
    pub download_dir: String,
    pub config_file: String,
    pub seen_file: String,
    pub auto_update: bool,
    pub telegram_token: String,
    pub telegram_chat_id: String,
}

pub fn upload(config: &AppConfig, file_path: &str, title: &str) -> Result<(), RiaError> {
    let upload_endpoint = format!("https://api.telegram.org/bot{}/sendAudio", &config.telegram_token);

    let song_file = MultipartPart::file(file_path)?
        .file_name(format!("{}.mp3", title));

    let form = MultipartForm::new()
        .part("audio", song_file)
        .text("chat_id", config.telegram_chat_id.clone())
        .text("title", title.to_string())
        .text("disable_notification", "true");

    debug!("Built multipart form for upload");

    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(&upload_endpoint)
        .multipart(form)
        .send();

    let resp = match resp {
        Ok(v) => { v }
        Err(mut e) => {
            // Hide token from error message
            if let Some(url) = e.url_mut() {
                let private_url = url.path()
                    .replace(
                        &config.telegram_token,
                        "X".repeat(config.telegram_token.len()).as_str());

                url.set_path(private_url.as_str())
            }

            return Err(RiaError::from(e))
        }
    };

    debug!("Sent the request");

    let json: Value = resp.json()?;

    debug!("Parsed response: {:?}", json);

    if json["ok"].as_bool() != Some(true) {
        return Err(
            RiaError::TelegramError(json["description"].as_str()
                .unwrap_or("No description field in json response")
                .to_string()))
    }

    Ok(())
}