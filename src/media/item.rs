//! Media item representation.

use chrono::{TimeZone, Utc};
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

        // Handle invalid timestamps gracefully with a fallback
        match Utc.timestamp_millis_opt(timestamp_ms) {
            chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%dT%H-%M-%S").to_string(),
            _ => format!("unknown_{}", self.created_at),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_item(created_at: i64, media_id: &str, is_preview: bool) -> MediaItem {
        MediaItem {
            media_id: media_id.to_string(),
            created_at,
            mimetype: "image/jpeg".to_string(),
            download_url: "https://example.com/image.jpg".to_string(),
            file_extension: "jpg".to_string(),
            resolution: 1920 * 1080,
            height: 1080,
            width: 1920,
            is_preview,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_timestamp_seconds_conversion() {
        // Timestamp in seconds (Jan 23, 2024 12:00:00 UTC)
        let item = create_test_item(1706011200, "123", false);
        let filename = item.generate_filename();
        // Should convert to milliseconds and format correctly
        assert!(filename.starts_with("2024-01-23T12-00-00"));
    }

    #[test]
    fn test_timestamp_milliseconds() {
        // Timestamp already in milliseconds (Jan 23, 2024 12:00:00 UTC)
        let item = create_test_item(1706011200000, "123", false);
        let filename = item.generate_filename();
        assert!(filename.starts_with("2024-01-23T12-00-00"));
    }

    #[test]
    fn test_timestamp_boundary() {
        // Just below the threshold (should be treated as seconds)
        let item = create_test_item(999_999_999_999, "123", false);
        let filename = item.generate_filename();
        // 999999999999 seconds * 1000 = year ~33658
        assert!(filename.contains("3365"));
    }

    #[test]
    fn test_filename_generation_regular() {
        let item = create_test_item(1706011200, "media123", false);
        let filename = item.generate_filename();
        assert_eq!(filename, "2024-01-23T12-00-00_id_media123.jpg");
    }

    #[test]
    fn test_filename_generation_preview() {
        let item = create_test_item(1706011200, "media123", true);
        let filename = item.generate_filename();
        assert_eq!(filename, "2024-01-23T12-00-00_preview_id_media123.jpg");
    }

    #[test]
    fn test_filename_with_hash() {
        let item = create_test_item(1706011200, "media123", false);
        let filename = item.generate_filename_with_hash("abc123def");
        assert_eq!(
            filename,
            "2024-01-23T12-00-00_id_media123_hash2_abc123def.jpg"
        );
    }

    #[test]
    fn test_media_type_detection() {
        let mut item = create_test_item(0, "123", false);

        item.mimetype = "image/jpeg".to_string();
        assert_eq!(item.media_type(), MediaType::Image);

        item.mimetype = "image/png".to_string();
        assert_eq!(item.media_type(), MediaType::Image);

        item.mimetype = "video/mp4".to_string();
        assert_eq!(item.media_type(), MediaType::Video);

        item.mimetype = "application/vnd.apple.mpegurl".to_string();
        assert_eq!(item.media_type(), MediaType::Video);

        item.mimetype = "audio/mpeg".to_string();
        assert_eq!(item.media_type(), MediaType::Audio);

        item.mimetype = "application/octet-stream".to_string();
        assert_eq!(item.media_type(), MediaType::Unknown);
    }

    #[test]
    fn test_is_m3u8() {
        let mut item = create_test_item(0, "123", false);

        item.mimetype = "application/vnd.apple.mpegurl".to_string();
        assert!(item.is_m3u8());

        item.mimetype = "video/mp4".to_string();
        item.download_url = "https://example.com/video.m3u8".to_string();
        assert!(item.is_m3u8());

        item.download_url = "https://example.com/video.mp4".to_string();
        assert!(!item.is_m3u8());
    }

    #[test]
    fn test_effective_extension() {
        let mut item = create_test_item(0, "123", false);
        item.file_extension = "jpg".to_string();
        assert_eq!(item.effective_extension(), "jpg");

        item.mimetype = "application/vnd.apple.mpegurl".to_string();
        assert_eq!(item.effective_extension(), "mp4");
    }

    #[test]
    fn test_folder_names() {
        assert_eq!(MediaType::Image.folder_name(), "Pictures");
        assert_eq!(MediaType::Video.folder_name(), "Videos");
        assert_eq!(MediaType::Audio.folder_name(), "Audio");
        assert_eq!(MediaType::Unknown.folder_name(), "Other");
    }
}
