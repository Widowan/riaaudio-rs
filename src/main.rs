mod errors;
mod feed;
mod utils;
mod youtube;
mod structs;

use std::fs::{self};
use std::{process, thread};
use std::time::Duration;
use log::{debug, error, info};
use structs::PipxOperation;
use structs::AppConfig;
use crate::errors::{RiaError, VideoError};
use crate::utils::manage_yt_dlp;

fn clear_start(download_dir: &String, seen_file: &String) {
    fs::remove_dir_all(download_dir).ok();
    fs::remove_file(seen_file).ok();
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    ctrlc::set_handler(|| {
        error!("Received SIGINT, exiting");
        process::exit(1);
    }).expect("Error setting SIGINT handler");

    let download_dir = std::env::var("RIA_DOWNLOAD_DIR").unwrap_or("downloads".to_string());
    let config_file = std::env::var("RIA_CONFIG").unwrap_or("config.yaml".to_string());
    let seen_file = std::env::var("RIA_SEEN_FILE").unwrap_or("seen_videos.txt".to_string());
    let auto_update = std::env::var("RIA_AUTO_UPDATE").map(|s| !s.is_empty()).unwrap_or(false);
    let telegram_token = std::env::var("RIA_TELEGRAM_TOKEN").unwrap_or("stub_token".to_string());
    let telegram_chat_id = std::env::var("RIA_TELEGRAM_CHAT_ID").unwrap_or("stub_chat_id".to_string());
    let sleep_timer = std::env::var("RIA_SLEEP_TIMER").unwrap_or("1800".to_string()).parse::<u64>().expect("sleep timer is NaN");

    let regexes = utils::setup_regexes();

    if cfg!(debug_assertions) {
        clear_start(&download_dir, &seen_file);
    }

    let app_config = AppConfig {
        download_dir,
        config_file,
        seen_file,
        auto_update,
        telegram_token,
        telegram_chat_id,
        regexes
    };

    fs::create_dir_all(&app_config.download_dir).expect("Failed to create downloads directory");

    let parser_config = match utils::load_config(&app_config) {
        Ok(config) => config,
        Err(e) => {
            error!("Config error: {}", e);
            process::exit(1);
        }
    };

    let (mut seen_ids, mut seen_titles) = utils::load_seen(&app_config);

    #[allow(clippy::collapsible_if)]
    if app_config.auto_update {
        if let Err(e) = manage_yt_dlp(PipxOperation::Install) {
            error!("Failed to install yt-dlp: {}", e);
        }
    }

    debug!("Loaded {} channels", parser_config.channels.len());

    loop {
        let mut download_error = false;

        for channel_id in &parser_config.channels {
            match youtube::process_channel(&app_config, channel_id, &mut seen_ids, &mut seen_titles) {
                Ok(_) => { }
                Err(RiaError::Video(VideoError::SeenVideo(v))) => {
                    info!("Skipped duplicate video: {}", v);
                }
                Err(RiaError::Video(VideoError::TitleValidationFailed(v))) => {
                    info!("Skipped video with failed title check: {}", v);
                }
                Err(RiaError::YtDlp(e)) => {
                    error!("{}", e);
                    download_error = true;
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        }

        if download_error && app_config.auto_update {
            error!("yt-dlp failed to download some videos, will try to update it just in case");
            if let Err(e) = manage_yt_dlp(PipxOperation::Upgrade) {
                error!("Failed to upgrade yt-dlp: {}", e);
            }
        }

        info!("Sleeping for {} seconds...", sleep_timer);
        thread::sleep(Duration::from_secs(sleep_timer));
    }
}