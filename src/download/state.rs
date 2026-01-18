//! Download state tracking.

use std::path::PathBuf;

use crate::config::DownloadType;
use crate::dedup::DedupService;
use crate::media::MediaType;

/// Per-creator download state.
#[derive(Debug, Default)]
pub struct DownloadState {
    // Creator info
    pub creator_name: Option<String>,
    pub creator_id: Option<String>,
    pub following: bool,
    pub subscribed: bool,

    // Paths
    pub base_path: Option<PathBuf>,
    pub download_path: Option<PathBuf>,

    // Current download type
    pub download_type: DownloadType,

    // Unified deduplication service
    pub dedup: DedupService,

    // Statistics
    pub pic_count: u64,
    pub vid_count: u64,
    pub audio_count: u64,
    pub total_message_items: u64,
    pub total_timeline_pictures: u64,
    pub total_timeline_videos: u64,
}

impl DownloadState {
    /// Create a new download state for a creator.
    pub fn new(creator_name: String, creator_id: String) -> Self {
        Self {
            creator_name: Some(creator_name),
            creator_id: Some(creator_id),
            ..Default::default()
        }
    }

    /// Get the creator ID or return an error.
    pub fn creator_id(&self) -> crate::error::Result<&str> {
        self.creator_id
            .as_deref()
            .ok_or_else(|| crate::error::Error::Api("Creator ID not set".into()))
    }

    /// Check if a media ID has already been seen.
    pub fn is_id_seen(&self, id: &str, media_type: MediaType) -> bool {
        self.dedup.is_id_seen(id, media_type)
    }

    /// Mark a media ID as seen.
    pub fn mark_id_seen(&mut self, id: String, media_type: MediaType) {
        self.dedup.mark_id_seen(id, media_type);
    }

    /// Check if a file hash has been seen.
    pub fn is_hash_seen(&self, hash: &str, media_type: MediaType) -> bool {
        self.dedup.is_hash_seen(hash, media_type)
    }

    /// Mark a hash as seen.
    pub fn mark_hash_seen(&mut self, hash: String, media_type: MediaType) {
        self.dedup.mark_hash_seen(hash, media_type);
    }

    /// Get the duplicate count.
    pub fn duplicate_count(&self) -> u64 {
        self.dedup.duplicates_found()
    }

    /// Increment duplicate count.
    pub fn increment_duplicate(&mut self) {
        self.dedup.record_duplicate();
    }

    /// Increment picture count.
    pub fn increment_pic(&mut self) {
        self.pic_count += 1;
    }

    /// Increment video count.
    pub fn increment_vid(&mut self) {
        self.vid_count += 1;
    }

    /// Increment audio count.
    pub fn increment_audio(&mut self) {
        self.audio_count += 1;
    }

    /// Get total downloaded count.
    pub fn total_downloaded(&self) -> u64 {
        self.pic_count + self.vid_count + self.audio_count
    }

    // Legacy compatibility methods - delegate to dedup service

    /// Check if a media ID has already been downloaded (image).
    pub fn is_photo_id_seen(&self, id: &str) -> bool {
        self.dedup.is_id_seen(id, MediaType::Image)
    }

    /// Check if a media ID has already been downloaded (video).
    pub fn is_video_id_seen(&self, id: &str) -> bool {
        self.dedup.is_id_seen(id, MediaType::Video)
    }

    /// Check if a media ID has already been downloaded (audio).
    pub fn is_audio_id_seen(&self, id: &str) -> bool {
        self.dedup.is_id_seen(id, MediaType::Audio)
    }

    /// Mark a photo media ID as seen.
    pub fn mark_photo_id_seen(&mut self, id: String) {
        self.dedup.mark_id_seen(id, MediaType::Image);
    }

    /// Mark a video media ID as seen.
    pub fn mark_video_id_seen(&mut self, id: String) {
        self.dedup.mark_id_seen(id, MediaType::Video);
    }

    /// Mark an audio media ID as seen.
    pub fn mark_audio_id_seen(&mut self, id: String) {
        self.dedup.mark_id_seen(id, MediaType::Audio);
    }

    /// Check if a file hash has been seen (image).
    pub fn is_photo_hash_seen(&self, hash: &str) -> bool {
        self.dedup.is_hash_seen(hash, MediaType::Image)
    }

    /// Check if a file hash has been seen (video).
    pub fn is_video_hash_seen(&self, hash: &str) -> bool {
        self.dedup.is_hash_seen(hash, MediaType::Video)
    }

    /// Check if a file hash has been seen (audio).
    pub fn is_audio_hash_seen(&self, hash: &str) -> bool {
        self.dedup.is_hash_seen(hash, MediaType::Audio)
    }

    /// Mark a photo hash as seen.
    pub fn mark_photo_hash_seen(&mut self, hash: String) {
        self.dedup.mark_hash_seen(hash, MediaType::Image);
    }

    /// Mark a video hash as seen.
    pub fn mark_video_hash_seen(&mut self, hash: String) {
        self.dedup.mark_hash_seen(hash, MediaType::Video);
    }

    /// Mark an audio hash as seen.
    pub fn mark_audio_hash_seen(&mut self, hash: String) {
        self.dedup.mark_hash_seen(hash, MediaType::Audio);
    }
}

/// Global statistics across all creators.
#[derive(Debug, Default)]
pub struct GlobalState {
    pub duplicate_count: u64,
    pub pic_count: u64,
    pub vid_count: u64,
    pub audio_count: u64,
    pub creators_processed: u64,
    pub creators_failed: u64,
}

impl GlobalState {
    /// Add statistics from a creator's download state.
    pub fn add_creator_stats(&mut self, state: &DownloadState) {
        self.duplicate_count += state.duplicate_count();
        self.pic_count += state.pic_count;
        self.vid_count += state.vid_count;
        self.audio_count += state.audio_count;
        self.creators_processed += 1;
    }

    /// Mark a creator as failed.
    pub fn mark_creator_failed(&mut self) {
        self.creators_failed += 1;
    }

    /// Get total downloaded count.
    pub fn total_downloaded(&self) -> u64 {
        self.pic_count + self.vid_count + self.audio_count
    }
}
