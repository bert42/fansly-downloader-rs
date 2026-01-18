//! Media item representation.

use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;

/// Type of media content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Unknown,
}

impl MediaType {
    /// Get the folder name for this media type.
    pub fn folder_name(&self) -> &'static str {
        match self {
            MediaType::Image => "Pictures",
            MediaType::Video => "Videos",
            MediaType::Audio => "Audio",
            MediaType::Unknown => "Other",
        }
    }
}

/// A downloadable media item.
#[derive(Debug, Clone)]
pub struct MediaItem {
    /// Unique media ID.
    pub media_id: String,

    /// Creation timestamp.
    pub created_at: i64,

    /// MIME type of the media.
    pub mimetype: String,

    /// Download URL.
    pub download_url: String,

    /// File extension (without dot).
    pub file_extension: String,

    /// Resolution (width * height).
    pub resolution: u64,

    /// Height in pixels.
    pub height: u32,

    /// Width in pixels.
    pub width: u32,

    /// Whether this is a preview.
    pub is_preview: bool,

    /// Additional metadata (e.g., CloudFront cookies for M3U8).
    pub metadata: HashMap<String, String>,
}

impl MediaItem {
    /// Get the media type based on MIME type.
    pub fn media_type(&self) -> MediaType {
        if self.mimetype.starts_with("image") {
            MediaType::Image
        } else if self.mimetype.starts_with("video") || self.mimetype.contains("mpegurl") {
            MediaType::Video
        } else if self.mimetype.starts_with("audio") {
            MediaType::Audio
        } else {
            MediaType::Unknown
        }
    }

    /// Check if this is an M3U8/HLS stream.
    pub fn is_m3u8(&self) -> bool {
        self.mimetype.contains("mpegurl") || self.download_url.contains(".m3u8")
    }

    /// Generate the filename for this media item.
    pub fn generate_filename(&self) -> String {
        let id_prefix = if self.is_preview { "preview_id" } else { "id" };
        let timestamp_str = self.format_timestamp();

        format!(
            "{}_{}_{}.{}",
            timestamp_str, id_prefix, self.media_id, self.file_extension
        )
    }

    /// Generate filename with hash included.
    pub fn generate_filename_with_hash(&self, hash: &str) -> String {
        let id_prefix = if self.is_preview { "preview_id" } else { "id" };
        let timestamp_str = self.format_timestamp();

        format!(
            "{}_{}_{}_hash2_{}.{}",
            timestamp_str, id_prefix, self.media_id, hash, self.file_extension
        )
    }

    /// Format the creation timestamp for filename.
    fn format_timestamp(&self) -> String {
        // API returns timestamps in milliseconds
        // For older content, timestamps might be in seconds (< year 2001 threshold)
        let timestamp_ms = if self.created_at < 1_000_000_000_000 {
            // Timestamp appears to be in seconds, convert to milliseconds
            self.created_at * 1000
        } else {
            self.created_at
        };
        let dt: DateTime<Utc> = Utc.timestamp_millis_opt(timestamp_ms).unwrap();
        dt.format("%Y-%m-%dT%H-%M-%S").to_string()
    }

    /// Get effective file extension, handling M3U8 → mp4 conversion.
    pub fn effective_extension(&self) -> &str {
        if self.is_m3u8() {
            "mp4"
        } else {
            &self.file_extension
        }
    }
}

impl Default for MediaItem {
    fn default() -> Self {
        Self {
            media_id: String::new(),
            created_at: 0,
            mimetype: String::new(),
            download_url: String::new(),
            file_extension: String::from("bin"),
            resolution: 0,
            height: 0,
            width: 0,
            is_preview: false,
            metadata: HashMap::new(),
        }
    }
}
