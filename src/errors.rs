use crate::structs::{ProcessOutput, VideoInfo};
use log::error;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum RiaError {
    #[error(transparent)]
    VideoError(#[from] VideoError),

    #[error(transparent)]
    PipxError(#[from] PipxError),

    #[error(transparent)]
    YtDlpError(#[from] YtDlpError),

    #[error(transparent)]
    FeedError(#[from] FeedError),

    #[error(transparent)]
    TelegramError(#[from] TelegramError),

    #[error("Failed to parse config: {0}")]
    ConfigParseError(#[from] serde_yaml::Error),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(ThisError, Debug)]
pub enum TelegramError {
    #[error("Error sending request to telegram: {0:?}")]
    RequestError(#[from] reqwest::Error),

    #[error("Unsuccessful telegram response: {0}")]
    ResponseError(String)
}

#[derive(ThisError, Debug)]
pub enum VideoError {
    #[error("Video was a short: {0}")]
    IsShort(VideoInfo),

    #[error("Title validation failed: {0}")]
    TitleValidationFailed(VideoInfo),

    #[error("Video is already seen: {0}")]
    SeenVideo(VideoInfo),
}

#[derive(ThisError, Debug)]
pub enum FeedError {
    #[error("Video ID tag not found in the RSS entry")]
    VideoIdNotFound,

    #[error("Error while parsing channel feed: {0}")]
    ParseError(#[from] feed_rs::parser::ParseFeedError),

    #[error("Error getting feed: {0:?}")]
    RequestError(#[from] reqwest::Error),

    #[error("Error in feed response: {0}")]
    ResponseError(String),

    #[error("RSS feed for the channel {0} is empty")]
    EmptyFeed(String),

    #[error("Missing field: {0}")]
    MissingField(String)
}

#[derive(ThisError, Debug)]
pub enum PipxError {
    #[error("Error calling pipx: {0}")]
    CallError(#[from] std::io::Error),

    #[error("Bad status returned by pipx:\n{0}")]
    StatusError(ProcessOutput),
}

#[derive(ThisError, Debug)]
pub enum YtDlpError {
    #[error("Error calling yt-dlp: {0}")]
    CallError(#[from] std::io::Error),

    #[error("Bad status returned by yt-dlp:\n{0}")]
    StatusError(ProcessOutput),
}

