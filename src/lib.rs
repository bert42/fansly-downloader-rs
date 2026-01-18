//! Fansly Downloader RS - A Rust implementation of fansly downloader
//!
//! This library provides functionality for downloading media content from Fansly creators.
//!
//! # Features
//!
//! - Download timeline posts
//! - Download direct messages
//! - Download single posts
//! - Download purchased collections
//! - Automatic deduplication via file hashing
//! - M3U8/HLS video support
//! - Rate limiting and retry logic
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use fansly_downloader::{Config, FanslyApi};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::load(Path::new("config.toml"))?;
//!     let api = FanslyApi::new(
//!         config.my_account.authorization_token.clone(),
//!         config.my_account.user_agent.clone(),
//!         config.my_account.check_key.clone(),
//!         config.cache.device_id.clone(),
//!         config.cache.device_id_timestamp,
//!     ).await?;
//!
//!     // ... download logic
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod cli;
pub mod config;
pub mod dedup;
pub mod download;
pub mod error;
pub mod fs;
pub mod media;
pub mod output;

// Re-exports for convenience
pub use api::FanslyApi;
pub use config::{Config, DownloadMode};
pub use download::{
    download_collections, download_messages, download_single_post, download_timeline,
    DownloadState, GlobalState,
};
pub use error::{Error, Result};
pub use media::{MediaItem, MediaType};
