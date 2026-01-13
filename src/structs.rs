use regex::Regex;
use serde::Deserialize;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, Deserialize)]
pub struct ParserConfig {
    pub channels: Vec<String>,
}

#[derive(Debug)]
pub enum PipxOperation {
    Install,
    Upgrade,
}

impl AsRef<str> for PipxOperation {
    fn as_ref(&self) -> &str {
       match self {
           PipxOperation::Install => "install",
           PipxOperation::Upgrade => "upgrade"
       }
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub download_dir: String,
    pub config_file: String,
    pub seen_file: String,
    pub auto_update: bool,
    pub telegram_token: String,
    pub telegram_chat_id: String,
    pub regexes: Regexes,
}

#[derive(Debug)]
pub struct Regexes {
    pub title_regex: Regex,
    pub bracket_regexes: Vec<Regex>,
    pub banned_regexes: Vec<Regex>,
    pub removed_regexes: Vec<Regex>,
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub title: String,
    pub id: String,
    pub url: String,
}

impl Display for VideoInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ id: {}, title: {} }}", self.id, self.title)
    }
}

#[derive(Debug)]
pub struct ProcessOutput {
    status: i32,
    stdout: String,
    stderr: String,
}

impl From<std::process::Output> for ProcessOutput {
    fn from(value: std::process::Output) -> Self {
        Self {
            status: value.status.code().unwrap_or(255),
            stdout: String::from_utf8_lossy(&value.stdout).to_string(),
            stderr: String::from_utf8_lossy(&value.stderr).to_string()
        }
    }
}

impl Display for ProcessOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "====EXIT CODE====\n{}====STDOUT====\n{}====STDERR====\n{}",
            self.status, self.stdout, self.stderr)
    }
}