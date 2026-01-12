use crate::errors::PipxError;
use crate::errors::RiaError;
use crate::errors::TelegramError;
use crate::structs::AppConfig;
use crate::structs::ParserConfig;
use crate::structs::PipxOperation;
use crate::structs::VideoInfo;
use log::debug;
use reqwest::blocking::multipart::Form as MultipartForm;
use reqwest::blocking::multipart::Part as MultipartPart;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::process::Command;

pub fn load_config(app_config: &AppConfig) -> Result<ParserConfig, RiaError> {
    let config_file = File::open(&app_config.config_file)?;
    Ok(serde_yaml::from_reader(config_file)?)
}

pub fn load_seen(app_config: &AppConfig) -> (HashSet<String>, HashSet<String>) {
    let mut seen_ids = HashSet::new();
    let mut seen_titles = HashSet::new();

    if let Ok(content) = fs::read_to_string(&app_config.seen_file) {
        for line in content.lines() {
            let trimmed_line = line.trim();
            if !trimmed_line.is_empty() {
                let (id, title) = trimmed_line.split_once("|").expect("Corrupted seen log file");
                seen_ids.insert(id.to_string());
                seen_titles.insert(title.to_string());
            }
        }
    }

    (seen_ids, seen_titles)
}

pub fn save_seen(
        app_config: &AppConfig,
        seen_ids: &mut HashSet<String>,
        seen_titles: &mut HashSet<String>,
        video_info: &VideoInfo
) -> Result<(), RiaError> {

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&app_config.seen_file)?;

    // Not returning immediately because it's not fatal and we need to save it to memory first
    let write_result = writeln!(file, "{}|{}", video_info.id, video_info.title);

    seen_ids.insert(video_info.id.clone());
    seen_titles.insert(video_info.title.clone());

    write_result?;

    Ok(())
}

pub fn upload(config: &AppConfig, file_path: &str, full_song_name: &str) -> Result<(), RiaError> {
    let upload_endpoint = format!("https://api.telegram.org/bot{}/sendAudio", &config.telegram_token);
    let (performer, title) = full_song_name.split_once(" - ")
        .expect("Unreachable, title already validated");

    let song_file = MultipartPart::file(file_path)?
        .file_name(format!("{}.mp3", full_song_name));

    let form = MultipartForm::new()
        .part("audio", song_file)
        .text("chat_id", config.telegram_chat_id.clone())
        .text("title", title.to_string())
        .text("performer", performer.to_string())
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

            return Err(TelegramError::from(e))?
        }
    };

    debug!("Sent the request");

    let json: Value = resp.json()
        .map_err(|_| TelegramError::ResponseError("Response json parsing error".to_string()))?;

    debug!("Parsed response: {:?}", json);

    if Some(Some(true)) != json.get("ok").map(|v| v.as_bool()) {
        return Err(TelegramError::ResponseError(json.to_string()))?
    }

    Ok(())
}

pub fn manage_yt_dlp(operation: PipxOperation) -> Result<(), PipxError> {
    let result = Command::new("pipx")
        .args([operation.as_ref(), "yt-dlp"])
        .output()?;

    debug!("pipx operation finished:\n====STDOUT====\n{}\n====STDERR====\n{}",
        String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr));

    if !result.status.success() {
        return Err(PipxError::StatusError(result.into()))
    }

    Ok(())
}