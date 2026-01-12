use std::collections::HashSet;
use std::fs;
use std::process::Command;
use log::{debug, info};
use regex::{Regex, RegexBuilder};
use crate::errors::RiaError;
use crate::errors::RiaError::TitleCheckFailed;
use crate::utils;
use crate::feed;
use crate::feed::VideoInfo;
use crate::utils::AppConfig;

fn download_audio(download_dir: &str, video_info: &VideoInfo) -> Result<String, RiaError> {
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
        .output()
        .map_err(|e| RiaError::YtDlpCallError(e))?;

    debug!("yt-dlp finished:\n====STDOUT====\n{}\n====STDERR====\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr));

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(RiaError::YtDlpStatusError(stderr.into_owned()));
    }

    let file_path = format!("{}/{}.mp3", download_dir, video_info.id);
    Ok(file_path)
}

fn validate_title(title: &str) -> Result<String, RiaError> {
    let mut title = title.to_string();

    let correct_title = Regex::new(r".* - .*").unwrap();
    let banned_regexes = [
        r"Best of",
        r"Best songs",
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
        return Err(TitleCheckFailed(correct_title.as_str().to_string()));
    }

    debug!("Title template validation passed");

    for regex in banned_regexes {
        if regex.is_match(&title) {
            return Err(TitleCheckFailed(regex.as_str().to_string()));
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
        debug!("No replacements were made");
    }

    Ok(title.to_string())
}

pub fn process_channel(app_config: &AppConfig, channel_id: &str, seen: &mut HashSet<String>) -> Result<(), RiaError> {
    let video_info = feed::get_latest_video_info(channel_id)?;

    if seen.contains(&video_info.id) {
        return Err(RiaError::DuplicateVideo);
    }

    let correct_title = validate_title(&video_info.title)?;

    if video_info.url.contains("/shorts/") {
        return Err(RiaError::IsShort(video_info.title))
    }

    let file_path = download_audio(&app_config.download_dir, &video_info)?;

    if let Err(e) = utils::upload(&app_config, &file_path, &correct_title) {
        fs::remove_file(&file_path)?;
        return Err(e)
    }

    info!("Upload successful");
    fs::remove_file(&file_path)?;
    debug!("File {} removed", file_path);

    utils::save_seen(&app_config, seen, &video_info.id)?;
    debug!("ID {} Saved", video_info.id);

    Ok(())
}