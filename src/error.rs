//! Error types for the fansly-downloader application.

use thiserror::Error;

/// Main error type for the application.
#[derive(Error, Debug)]
pub enum Error {
    // Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid configuration value for '{field}': {message}")]
    ConfigValidation { field: String, message: String },

    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    // API errors
    #[error("API error: {0}")]
    Api(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    // Download errors
    #[error("Download failed: {0}")]
    Download(String),

    #[error("M3U8 processing error: {0}")]
    M3U8(String),

    #[error("Duplicate threshold exceeded ({0} duplicates)")]
    DuplicateThreshold(u64),

    // File system errors
    #[error("Invalid filename (path traversal attempt): {0}")]
    InvalidFilename(String),

    // Media errors
    #[error("Invalid media: {0}")]
    Media(String),

    #[error("Invalid MP4 file: {0}")]
    InvalidMp4(String),

    // External tool errors
    #[error("FFmpeg error: {0}")]
    FFmpeg(String),

    #[error("FFmpeg not found. Please install ffmpeg and ensure it's in your PATH.")]
    FFmpegNotFound,

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // HTTP errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    // WebSocket errors (boxed to reduce enum size)
    #[error("WebSocket error: {0}")]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),

    // Serialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    // URL parsing errors
    #[error("Invalid URL: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// Result type alias using our Error type.
pub type Result<T> = std::result::Result<T, Error>;

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Error::WebSocket(Box::new(err))
    }
}

/// Exit codes matching the Python implementation.
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const ABORT: i32 = 1;
    pub const API_ERROR: i32 = 2;
    pub const CONFIG_ERROR: i32 = 3;
    pub const DOWNLOAD_ERROR: i32 = 4;
    pub const UNEXPECTED_ERROR: i32 = 5;
    pub const SOME_USERS_FAILED: i32 = 6;
}
