//! Configuration module for the fansly-downloader.
//!
//! This module handles:
//! - Loading configuration from TOML files
//! - CLI argument parsing and merging
//! - Configuration validation

pub mod loader;
pub mod modes;
pub mod validation;

pub use loader::{AccountConfig, CacheConfig, Config, CreatorConfig, OptionsConfig};
pub use modes::{DownloadMode, DownloadType};
pub use validation::{parse_post_id, validate_config};
