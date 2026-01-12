mod errors;
mod feed;
mod utils;
mod youtube;

use std::fs::{self};
use std::{process, thread};
use std::time::Duration;
use log::{debug, error, info};
use utils::AppConfig;
use crate::errors::RiaError;
use crate::utils::{manage_yt_dlp, PipxOperation};

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

    if cfg!(debug_assertions) {
        clear_start(&download_dir, &seen_file);
    }

    let app_config = AppConfig {
        download_dir,
        config_file,
        seen_file,
        auto_update,
        telegram_token,
        telegram_chat_id
    };

    fs::create_dir_all(&app_config.download_dir).expect("Failed to create downloads directory");

    let parser_config = match utils::load_config(&app_config) {
        Ok(config) => config,
        Err(e) => {
            error!("Config error: {}", e);
            process::exit(1);
        }
    };

    let mut seen = utils::load_seen(&app_config);

    if app_config.auto_update {
        if let Err(e) = manage_yt_dlp(PipxOperation::Install) {
            error!("Failed to install yt-dlp: {}", e);
        }
    }

    debug!("Loaded {} channels", parser_config.channels.len());

    loop {
        let mut downloaded_any = false;
        let mut all_seen = true;

        for channel_id in &parser_config.channels {
            match youtube::process_channel(&app_config, channel_id, &mut seen) {
                Ok(_) => {
                    downloaded_any = true;
                    all_seen = false;
                },
                Err(RiaError::YtDlpCallError(e)) => {
                    error!("Failed to call yt-dlp: {}", e);
                }
                Err(e) => {
                    error!("{:?}: {}", e, e);
                    all_seen = false;
                }
            }
        }

        if !downloaded_any && !all_seen && app_config.auto_update {
            error!("yt-dlp failed to download all videos, will try to update it just in case");
            if let Err(e) = manage_yt_dlp(PipxOperation::Upgrade) {
                error!("Failed to upgrade yt-dlp: {}", e);
            }
        }

        info!("Sleeping for 30 minutes...");
        thread::sleep(Duration::from_secs(30 * 60));
    }
}