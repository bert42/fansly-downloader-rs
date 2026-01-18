//! Configuration structures and loading logic.

use crate::config::modes::DownloadMode;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub targeted_creator: CreatorConfig,

    pub my_account: AccountConfig,

    #[serde(default)]
    pub options: OptionsConfig,

    #[serde(default)]
    pub cache: CacheConfig,
}

/// Creator targeting configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreatorConfig {
    /// List of creator usernames to download from.
    #[serde(default)]
    pub usernames: HashSet<String>,
}

/// Account credentials configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// Fansly authorization token.
    pub authorization_token: String,

    /// Browser user agent string.
    #[serde(default = "default_user_agent")]
    pub user_agent: String,

    /// Fansly check key for request signing.
    #[serde(default = "default_check_key")]
    pub check_key: String,
}

/// Download options configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsConfig {
    /// Download mode (normal, timeline, messages, single, collection).
    #[serde(default)]
    pub download_mode: DownloadMode,

    /// Base directory for downloads.
    #[serde(default)]
    pub download_directory: Option<PathBuf>,

    /// Whether to download media previews.
    #[serde(default = "default_true")]
    pub download_media_previews: bool,

    /// Whether to separate messages into their own folder.
    #[serde(default = "default_true")]
    pub separate_messages: bool,

    /// Whether to separate timeline into its own folder.
    #[serde(default = "default_true")]
    pub separate_timeline: bool,

    /// Whether to separate previews into their own folder.
    #[serde(default)]
    pub separate_previews: bool,

    /// Whether to add "_fansly" suffix to creator folders.
    #[serde(default = "default_true")]
    pub use_folder_suffix: bool,

    /// Whether to show download progress.
    #[serde(default = "default_true")]
    pub show_downloads: bool,

    /// Whether to show skipped downloads.
    #[serde(default = "default_true")]
    pub show_skipped_downloads: bool,

    /// Whether to use duplicate threshold to stop early.
    #[serde(default)]
    pub use_duplicate_threshold: bool,

    /// Number of timeline retry attempts on empty response.
    #[serde(default = "default_timeline_retries")]
    pub timeline_retries: u32,

    /// Seconds to wait between timeline retries.
    #[serde(default = "default_timeline_delay")]
    pub timeline_delay_seconds: u64,

    /// Post ID for single post download mode.
    #[serde(default)]
    pub single_post_id: Option<String>,
}

impl Default for OptionsConfig {
    fn default() -> Self {
        Self {
            download_mode: DownloadMode::default(),
            download_directory: None,
            download_media_previews: true,
            separate_messages: true,
            separate_timeline: true,
            separate_previews: false,
            use_folder_suffix: true,
            show_downloads: true,
            show_skipped_downloads: true,
            use_duplicate_threshold: false,
            timeline_retries: 1,
            timeline_delay_seconds: 10,
            single_post_id: None,
        }
    }
}

/// Cached values configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Device ID from browser (fansly-d cookie value).
    /// This is required for authentication.
    pub device_id: Option<String>,

    /// Timestamp when device ID was obtained.
    pub device_id_timestamp: Option<i64>,
}

fn default_check_key() -> String {
    "qybZy9-fyszis-bybxyf".to_string()
}

fn default_user_agent() -> String {
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36".to_string()
}

fn default_true() -> bool {
    true
}

fn default_timeline_retries() -> u32 {
    1
}

fn default_timeline_delay() -> u64 {
    60
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::Config(format!(
                    "Configuration file not found: {}. Create one from config.example.toml",
                    path.display()
                ))
            } else {
                Error::Io(e)
            }
        })?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get the effective download directory.
    pub fn download_directory(&self) -> PathBuf {
        self.options
            .download_directory
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Update cache values and save to file if path provided.
    pub fn update_cache(
        &mut self,
        device_id: String,
        timestamp: i64,
        path: Option<&Path>,
    ) -> Result<()> {
        self.cache.device_id = Some(device_id);
        self.cache.device_id_timestamp = Some(timestamp);

        if let Some(path) = path {
            self.save(path)?;
        }

        Ok(())
    }
}

impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            authorization_token: String::new(),
            user_agent: default_user_agent(),
            check_key: default_check_key(),
        }
    }
}
