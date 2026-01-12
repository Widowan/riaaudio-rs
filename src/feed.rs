use feed_rs::model::{Feed, Link};
use url::Url;
use feed_rs::parser::parse;
use log::{info, warn};
use crate::errors::RiaError;

pub struct VideoInfo {
    pub title: String,
    pub id: String,
    pub url: String,
}

fn fetch_feed(channel_id: &str) -> Result<Feed, RiaError> {
    let rss_url = format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", channel_id);
    let response = reqwest::blocking::get(&rss_url)?;
    if !response.status().is_success() {
        return Err(RiaError::HttpResponseCodeError(
            format!("HTTP code {} for channel {}", response.status(), channel_id)
        ));
    }
    let body = response.bytes()?;
    let feed = parse(body.as_ref())?;
    Ok(feed)
}

fn extract_video_id(entry: feed_rs::model::Entry) -> Result<(String, String), RiaError> {
    if let Some(alt_link) = entry.links.into_iter().find(|l: &Link| l.rel.as_deref() == Some("alternate")) {
        let href = alt_link.href;

        if let Ok(parsed) = Url::parse(&href) {
            if let Some(video_id) = parsed.query_pairs()
                .find(|(key, _)| key == "v")
                .map(|(_, value)| value.to_string())
            {
                return Ok((href, video_id));
            }
        }
    }

    Err(RiaError::VideoIdNotFound)
}

pub fn get_latest_video_info(channel_id: &str) -> Result<VideoInfo, RiaError> {
    let feed = fetch_feed(channel_id)?;

    // TODO: ??
    let channel_name = feed.title
        .as_ref()
        .and_then(|t| Some(t.content.to_string()))
        .unwrap_or("Unknown channel".to_string());

    info!("Checking channel feed: {} ({})", channel_name, channel_id);

    let entries = feed.entries;
    if entries.is_empty() {
        warn!("No videos found in feed for channel {}", channel_id);
        return Err(RiaError::EmptyFeed);
    }

    let latest_entry = &entries[0];

    // TODO: ??
    let title = latest_entry.title
        .as_ref()
        .and_then(|t| Some(t.content.to_string()))
        .unwrap_or("Unknown title".to_string());

    let (video_url, video_id) = extract_video_id(latest_entry.clone())?;
    info!("New video: {} (ID: {})", title, video_id);

    Ok(VideoInfo { title, id: video_id, url: video_url })
}