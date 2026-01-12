use std::collections::HashSet;
use std::fs;
use std::process::Command;
use log::{debug, info};
use regex::{Regex, RegexBuilder};
use crate::errors::{RiaError, VideoError, YtDlpError};
use crate::utils;
use crate::feed;
use crate::structs::{AppConfig, VideoInfo};

fn download(download_dir: &str, video_info: &VideoInfo) -> Result<String, YtDlpError> {
    let output_template = format!("{}/{}.mp3", download_dir, video_info.id);

    info!("Downloading audio: {} ({})", video_info.title, video_info.id);

    let result = Command::new("yt-dlp")
        .args([
            "-x", "-f", "bestaudio",
            "--audio-format", "mp3",
            "--no-playlist",
            "-o", &output_template,
            &video_info.url,
        ])
        .output()?;

    if !result.status.success() {
        return Err(YtDlpError::StatusError(result.into()));
    }

    let file_path = format!("{}/{}.mp3", download_dir, video_info.id);
    Ok(file_path)
}

fn validate_title(video_info: &VideoInfo) -> Result<String, VideoError> {
    let mut title = video_info.title.clone();

    let correct_title = Regex::new(r".* - .*").unwrap();
    let banned_regexes = [
        r"Best of",
        r"Best song",
        r" Mix",
        r"Recap",
        r"Album",
    ].map(|r| RegexBuilder::new(r).case_insensitive(true).build().unwrap());
    let bracket_pairs = [
        (r"\(", r"\)"),
        (r"\{", r"\}"),
        (r"\[", r"\]"),

        (r"【", r"】"),
        (r"﹝", r"﹞"),
        (r"❨", r"❩"),
        (r"❪", r"❫"),
        (r"⟨", r"⟩"),
        (r"❮", r"❯"),
        (r"❰", r"❱"),
        (r"⁅", r"⁆"),
        (r"❬", r"❭"),
        (r"⦗", r"⦘"),
        (r"❲", r"❳")
    ];
    let removed_regexes = [
        r"Lyric",
        r"Official",
        r"Visualizer",
        r"Visualiser",
        r"Release",
        r"Video",
        r"Monstercat",
        r"Music"
    ].iter()
        .map(|re| {
            // There probably exists better solution than generating M*N regexes, but I can't be bothered
            bracket_pairs.iter()
                .map(|&(l, r)| format!(" ?{}.*?{}.*?{}", l, re, r))
                .collect::<Vec<String>>()
        })
        .flatten()
        .map(|re| RegexBuilder::new(&re).case_insensitive(true).build().unwrap())
        .collect::<Vec<Regex>>();

    debug!("Validating title: {}", title);

    if !correct_title.is_match(&title) {
        return Err(VideoError::TitleValidationFailed(video_info.clone()));
    }

    debug!("Title template validation passed");

    for regex in banned_regexes {
        if regex.is_match(&title) {
            return Err(VideoError::TitleValidationFailed(video_info.clone()));
        }
    }

    debug!("No banned regexes found");

    let mut replaced = false;
    for regex in removed_regexes {
        title = regex.replace_all(&title, |caps: &regex::Captures| {
            replaced = true;
            let m = caps.get(0).unwrap();
            debug!("Removed part: {}", m.as_str());
            return ""
        }).to_string();
    }

    if !replaced {
        debug!("Title was not modified");
    }

    Ok(title.to_string())
}

pub fn process_channel(
    app_config: &AppConfig,
    channel_id: &str,
    seen_ids: &mut HashSet<String>,
    seen_titles: &mut HashSet<String>
) -> Result<(), RiaError> {
    let video_info = feed::get_latest_video_info(channel_id)?;

    if video_info.url.contains("/shorts/") {
        return Err(VideoError::IsShort(video_info))?
    }

    if seen_ids.contains(&video_info.id) {
        return Err(VideoError::SeenVideo(video_info))?
    }

    let validated_title = validate_title(&video_info)?;

    let video_info = VideoInfo {
        title: validated_title,
        id: video_info.id.clone(),
        url: video_info.url
    };

    if seen_titles.contains(&video_info.title) {
        return Err(VideoError::SeenVideo(video_info))?
    }

    let file_path = download(&app_config.download_dir, &video_info)?;

    if let Err(e) = utils::upload(&app_config, &file_path, &video_info.title) {
        fs::remove_file(&file_path)?;
        return Err(e)
    }

    info!("Upload successful");
    fs::remove_file(&file_path)?;
    debug!("File {} removed", file_path);

    utils::save_seen(&app_config, seen_ids, seen_titles, &video_info)?;
    debug!("ID {} Saved", video_info.id);

    Ok(())
}

