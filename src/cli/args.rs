//! Command-line argument definitions using clap.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use crate::config::{Config, DownloadMode};

/// Fansly content downloader CLI.
#[derive(Parser, Debug)]
#[command(
    name = "fansly-downloader",
    version,
    about = "Download media content from Fansly creators",
    long_about = "A CLI tool to download photos, videos, and audio from Fansly creators.\n\n\
                  Supports downloading from timeline, messages, single posts, and purchased collections."
)]
pub struct Args {
    /// Creator username(s) to download from.
    /// Can specify multiple users separated by spaces.
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    pub user: Option<Vec<String>>,

    /// Base directory for downloads.
    #[arg(short = 'd', long = "directory")]
    pub download_directory: Option<PathBuf>,

    /// Fansly authorization token.
    #[arg(short, long, env = "FANSLY_TOKEN")]
    pub token: Option<String>,

    /// Browser user agent string.
    #[arg(short = 'a', long = "user-agent", env = "FANSLY_USER_AGENT")]
    pub user_agent: Option<String>,

    /// Fansly check key for request signing.
    #[arg(short = 'k', long = "check-key", env = "FANSLY_CHECK_KEY")]
    pub check_key: Option<String>,

    /// Device ID (from fansly-d cookie or Fansly-Client-Id header).
    #[arg(long = "device-id", env = "FANSLY_DEVICE_ID")]
    pub device_id: Option<String>,

    /// Download mode.
    #[arg(long, value_enum)]
    pub mode: Option<DownloadModeArg>,

    /// Post ID or URL for single post download (requires --mode single).
    #[arg(long)]
    pub post: Option<String>,

    /// Path to configuration file.
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    /// Don't add "_fansly" suffix to creator folders.
    #[arg(long)]
    pub no_folder_suffix: bool,

    /// Don't download preview media.
    #[arg(long)]
    pub no_previews: bool,

    /// Hide download progress information.
    #[arg(long, short)]
    pub quiet: bool,

    /// Show information about skipped downloads.
    #[arg(long)]
    pub show_skipped: bool,

    /// Use duplicate threshold to stop downloading early.
    /// Stops after encountering too many duplicates.
    #[arg(long)]
    pub use_duplicate_threshold: bool,

    /// Number of retry attempts for empty timeline responses.
    #[arg(long)]
    pub timeline_retries: Option<u32>,

    /// Seconds to wait between timeline retry attempts.
    #[arg(long)]
    pub timeline_delay: Option<u64>,

    /// Enable debug logging.
    #[arg(long)]
    pub debug: bool,
}

/// CLI download mode argument.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DownloadModeArg {
    /// Download both timeline and messages.
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

impl From<DownloadModeArg> for DownloadMode {
    fn from(arg: DownloadModeArg) -> Self {
        match arg {
            DownloadModeArg::Normal => DownloadMode::Normal,
            DownloadModeArg::Timeline => DownloadMode::Timeline,
            DownloadModeArg::Messages => DownloadMode::Messages,
            DownloadModeArg::Single => DownloadMode::Single,
            DownloadModeArg::Collection => DownloadMode::Collection,
        }
    }
}

impl Args {
    /// Merge CLI arguments into an existing config, overriding where specified.
    pub fn merge_into_config(self, config: &mut Config) {
        // Override usernames if provided
        if let Some(users) = self.user {
            config.targeted_creator.usernames = users.into_iter().collect();
        }

        // Override account settings if provided
        if let Some(token) = self.token {
            config.my_account.authorization_token = token;
        }

        if let Some(user_agent) = self.user_agent {
            config.my_account.user_agent = user_agent;
        }

        if let Some(check_key) = self.check_key {
            config.my_account.check_key = check_key;
        }

        if let Some(device_id) = self.device_id {
            config.cache.device_id = Some(device_id);
        }

        // Override options if provided
        if let Some(dir) = self.download_directory {
            config.options.download_directory = Some(dir);
        }

        if let Some(mode) = self.mode {
            config.options.download_mode = mode.into();
        }

        if let Some(post) = self.post {
            config.options.single_post_id = Some(post);
        }

        // Boolean flags (only override if set to non-default)
        if self.no_folder_suffix {
            config.options.use_folder_suffix = false;
        }

        if self.no_previews {
            config.options.download_media_previews = false;
        }

        if self.quiet {
            config.options.show_downloads = false;
            config.options.show_skipped_downloads = false;
        }

        if self.show_skipped {
            config.options.show_skipped_downloads = true;
        }

        if self.use_duplicate_threshold {
            config.options.use_duplicate_threshold = true;
        }

        if let Some(retries) = self.timeline_retries {
            config.options.timeline_retries = retries;
        }

        if let Some(delay) = self.timeline_delay {
            config.options.timeline_delay_seconds = delay;
        }
    }
}
