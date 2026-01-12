use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum RiaError {
    #[error("Video was a short")]
    IsShort(String),
    #[error("Title validation failed")]
    TitleCheckFailed(String),
    #[error("Failed to parse config")]
    ConfigParseError(#[from] serde_yaml::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("HTTP Error")]
    HttpResponseCodeError(String),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Video ID not found in the RSS entry")]
    VideoIdNotFound,
    #[error("Feed parsing error")]
    FeedParseError(#[from] feed_rs::parser::ParseFeedError),
    #[error("RSS feed has no entries")]
    EmptyFeed,
    #[error("Video is a duplicate")]
    DuplicateVideo,
    #[error("yt-dlp failed")]
    YtDlpStatusError(String),
    #[error("Failed to call yt-dlp")]
    YtDlpCallError(std::io::Error),
    #[error("Telegram sending error")]
    TelegramError(String),
    #[error("pipx call error")]
    PipxCallError(std::io::Error),
    #[error("pipx failed")]
    PipxStatusError(String),
}