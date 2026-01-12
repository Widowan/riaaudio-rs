use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum RiaError {
    #[error("Video was a short: {0}")]
    IsShort(String),
    #[error("Title validation failed: {0}")]
    TitleCheckFailed(String),
    #[error("Failed to parse config: {0}")]
    ConfigParseError(#[from] serde_yaml::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("HTTP Error: {0}")]
    HttpResponseCodeError(String),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Video ID not found in the RSS entry")]
    VideoIdNotFound,
    #[error("Feed parsing error: {0}")]
    FeedParseError(#[from] feed_rs::parser::ParseFeedError),
    #[error("RSS feed has no entries")]
    EmptyFeed,
    #[error("Video is a duplicate")]
    DuplicateVideo,
    #[error("yt-dlp failed: {0}")]
    YtDlpStatusError(String),
    #[error("Failed to call yt-dlp: {0}")]
    YtDlpCallError(std::io::Error),
    #[error("Telegram sending error: {0}")]
    TelegramError(String),
    #[error("pipx call error: {0}")]
    PipxCallError(std::io::Error),
    #[error("pipx failed: {0}")]
    PipxStatusError(String),
}