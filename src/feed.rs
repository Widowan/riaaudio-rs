use crate::errors::FeedError;
use crate::structs::VideoInfo;
use feed_rs::model::Feed;
use feed_rs::model::Link;
use feed_rs::parser::parse;
use log::info;
use url::Url;

fn fetch_feed(channel_id: &str) -> Result<Feed, FeedError> {
    let rss_url = format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", channel_id);
    let response = reqwest::blocking::get(&rss_url)?;
    if !response.status().is_success() {
        return Err(FeedError::ResponseError(
            format!("response {} for channel {}", response.status().as_u16(), channel_id)))
    }
    let body = response.bytes()?;
    let feed = parse(body.as_ref())?;
    Ok(feed)
}

fn extract_video_id(entry: feed_rs::model::Entry) -> Result<(String, String), FeedError> {
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

    Err(FeedError::VideoIdNotFound)
}

pub fn get_latest_video_info(channel_id: &str) -> Result<VideoInfo, FeedError> {
    let feed = fetch_feed(channel_id)?;

    let channel_name = feed.title
        .as_ref()
        .map(|t| t.content.to_string())
        .ok_or(FeedError::MissingField("channel name".to_string()))?;

    info!("Checking channel feed: {} ({})", channel_name, channel_id);

    let entries = feed.entries;
    if entries.is_empty() {
        return Err(FeedError::EmptyFeed(channel_id.to_string()))
    }

    let latest_entry = &entries[0];

    let title = latest_entry.title
        .as_ref()
        .map(|t| t.content.to_string())
        .ok_or(FeedError::MissingField("video title".to_string()))?;

    let (video_url, video_id) = extract_video_id(latest_entry.clone())?;
    info!("Newest video: {} (ID: {})", title, video_id);

    Ok(VideoInfo { title, id: video_id, url: video_url })
}