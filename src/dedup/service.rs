//! Unified deduplication service.
//!
//! This service abstracts both in-memory tracking and file system scanning
//! for duplicate detection.

use std::collections::HashSet;
use std::path::Path;

use crate::dedup::hash::{extract_hash_from_filename, hash_file};
use crate::error::Result;
use crate::media::MediaType;

/// Unified deduplication service that handles both media ID and hash-based deduplication.
#[derive(Debug, Default)]
pub struct DedupService {
    // Media ID tracking
    photo_media_ids: HashSet<String>,
    video_media_ids: HashSet<String>,
    audio_media_ids: HashSet<String>,

    // Hash tracking
    photo_hashes: HashSet<String>,
    video_hashes: HashSet<String>,
    audio_hashes: HashSet<String>,

    // Statistics
    duplicates_found: u64,
}

impl DedupService {
    /// Create a new deduplication service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan a directory for existing files and populate tracking sets.
    pub fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            self.index_file(&path);
        }

        Ok(())
    }

    /// Index a single file (extract media ID and/or hash from filename).
    fn index_file(&mut self, path: &Path) {
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return,
        };

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let media_type = extension_to_media_type(extension);
        if matches!(media_type, MediaType::Unknown) {
            return;
        }

        // Try to extract hash from filename first
        if let Some(hash) = extract_hash_from_filename(filename) {
            self.mark_hash_seen(hash, media_type);
            return;
        }

        // Try to extract media ID from filename
        // Pattern: {timestamp}_{id|preview_id}_{media_id}.{ext}
        if let Some(media_id) = extract_media_id_from_filename(filename) {
            self.mark_id_seen(media_id, media_type);
        }
    }

    /// Check if a media ID has been seen.
    pub fn is_id_seen(&self, id: &str, media_type: MediaType) -> bool {
        match media_type {
            MediaType::Image => self.photo_media_ids.contains(id),
            MediaType::Video => self.video_media_ids.contains(id),
            MediaType::Audio => self.audio_media_ids.contains(id),
            MediaType::Unknown => false,
        }
    }

    /// Mark a media ID as seen.
    pub fn mark_id_seen(&mut self, id: String, media_type: MediaType) {
        match media_type {
            MediaType::Image => self.photo_media_ids.insert(id),
            MediaType::Video => self.video_media_ids.insert(id),
            MediaType::Audio => self.audio_media_ids.insert(id),
            MediaType::Unknown => false,
        };
    }

    /// Check if a hash has been seen.
    pub fn is_hash_seen(&self, hash: &str, media_type: MediaType) -> bool {
        match media_type {
            MediaType::Image => self.photo_hashes.contains(hash),
            MediaType::Video => self.video_hashes.contains(hash),
            MediaType::Audio => self.audio_hashes.contains(hash),
            MediaType::Unknown => false,
        }
    }

    /// Mark a hash as seen.
    pub fn mark_hash_seen(&mut self, hash: String, media_type: MediaType) {
        match media_type {
            MediaType::Image => self.photo_hashes.insert(hash),
            MediaType::Video => self.video_hashes.insert(hash),
            MediaType::Audio => self.audio_hashes.insert(hash),
            MediaType::Unknown => false,
        };
    }

    /// Check if a file is a duplicate by computing and checking its hash.
    pub fn is_file_duplicate(&self, path: &Path, media_type: MediaType) -> Result<bool> {
        let hash = hash_file(path, media_type)?;
        Ok(self.is_hash_seen(&hash, media_type))
    }

    /// Add a file's hash to tracking and return the hash.
    pub fn add_file_hash(&mut self, path: &Path, media_type: MediaType) -> Result<String> {
        let hash = hash_file(path, media_type)?;
        self.mark_hash_seen(hash.clone(), media_type);
        Ok(hash)
    }

    /// Record a duplicate was found.
    pub fn record_duplicate(&mut self) {
        self.duplicates_found += 1;
    }

    /// Get the number of duplicates found.
    pub fn duplicates_found(&self) -> u64 {
        self.duplicates_found
    }

    /// Get total tracked items count.
    pub fn tracked_count(&self) -> usize {
        self.photo_media_ids.len()
            + self.video_media_ids.len()
            + self.audio_media_ids.len()
            + self.photo_hashes.len()
            + self.video_hashes.len()
            + self.audio_hashes.len()
    }
}

/// Convert file extension to MediaType.
fn extension_to_media_type(ext: &str) -> MediaType {
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" => MediaType::Image,
        "mp4" | "webm" | "mov" => MediaType::Video,
        "mp3" | "m4a" | "ogg" | "wav" => MediaType::Audio,
        _ => MediaType::Unknown,
    }
}

/// Extract media ID from filename.
/// Pattern: {timestamp}_{id|preview_id}_{media_id}.{ext}
fn extract_media_id_from_filename(filename: &str) -> Option<String> {
    let parts: Vec<&str> = filename.split('_').collect();
    if parts.len() < 3 {
        return None;
    }

    // Try to find the media ID (should be the last numeric part before extension)
    for part in parts.iter().rev() {
        if let Some(dot_pos) = part.find('.') {
            let potential_id = &part[..dot_pos];
            if potential_id.chars().all(|c| c.is_ascii_digit()) && potential_id.len() > 5 {
                return Some(potential_id.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_to_media_type() {
        assert_eq!(extension_to_media_type("jpg"), MediaType::Image);
        assert_eq!(extension_to_media_type("JPEG"), MediaType::Image);
        assert_eq!(extension_to_media_type("mp4"), MediaType::Video);
        assert_eq!(extension_to_media_type("mp3"), MediaType::Audio);
        assert_eq!(extension_to_media_type("txt"), MediaType::Unknown);
    }

    #[test]
    fn test_extract_media_id_from_filename() {
        assert_eq!(
            extract_media_id_from_filename("2024-01-01T12-00-00_id_1234567890.jpg"),
            Some("1234567890".to_string())
        );
        assert_eq!(
            extract_media_id_from_filename("2024-01-01T12-00-00_preview_id_1234567890.jpg"),
            Some("1234567890".to_string())
        );
        assert_eq!(extract_media_id_from_filename("random.jpg"), None);
    }

    #[test]
    fn test_dedup_service_id_tracking() {
        let mut service = DedupService::new();

        assert!(!service.is_id_seen("123", MediaType::Image));
        service.mark_id_seen("123".to_string(), MediaType::Image);
        assert!(service.is_id_seen("123", MediaType::Image));
        assert!(!service.is_id_seen("123", MediaType::Video));
    }

    #[test]
    fn test_dedup_service_hash_tracking() {
        let mut service = DedupService::new();

        assert!(!service.is_hash_seen("abc123", MediaType::Video));
        service.mark_hash_seen("abc123".to_string(), MediaType::Video);
        assert!(service.is_hash_seen("abc123", MediaType::Video));
        assert!(!service.is_hash_seen("abc123", MediaType::Image));
    }

    #[test]
    fn test_duplicate_counting() {
        let mut service = DedupService::new();

        assert_eq!(service.duplicates_found(), 0);
        service.record_duplicate();
        service.record_duplicate();
        assert_eq!(service.duplicates_found(), 2);
    }
}
