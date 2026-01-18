//! Download mode definitions.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Available download modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadMode {
    /// Download both timeline and messages (default).
    #[default]
    Normal,
    /// Download only timeline posts.
    Timeline,
    /// Download only direct messages.
    Messages,
    /// Download a single post by ID.
    Single,
    /// Download purchased media collections.
    Collection,
}

impl fmt::Display for DownloadMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadMode::Normal => write!(f, "normal"),
            DownloadMode::Timeline => write!(f, "timeline"),
            DownloadMode::Messages => write!(f, "messages"),
            DownloadMode::Single => write!(f, "single"),
            DownloadMode::Collection => write!(f, "collection"),
        }
    }
}

impl FromStr for DownloadMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(DownloadMode::Normal),
            "timeline" => Ok(DownloadMode::Timeline),
            "messages" => Ok(DownloadMode::Messages),
            "single" => Ok(DownloadMode::Single),
            "collection" => Ok(DownloadMode::Collection),
            _ => Err(format!("Unknown download mode: {}", s)),
        }
    }
}

/// Type of content currently being downloaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadType {
    #[default]
    NotSet,
    Timeline,
    Messages,
    Single,
    Collections,
}

impl fmt::Display for DownloadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadType::NotSet => write!(f, "not set"),
            DownloadType::Timeline => write!(f, "timeline"),
            DownloadType::Messages => write!(f, "messages"),
            DownloadType::Single => write!(f, "single"),
            DownloadType::Collections => write!(f, "collections"),
        }
    }
}
