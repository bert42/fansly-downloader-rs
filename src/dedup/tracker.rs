//! Deduplication tracking.

use std::path::Path;

use crate::dedup::hash::{extract_hash_from_filename, hash_file};
use crate::download::DownloadState;
use crate::error::Result;
use crate::media::MediaType;

/// Scan a directory for existing files and populate the download state with their hashes.
pub fn scan_existing_files(dir: &Path, state: &mut DownloadState) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Try to extract hash from filename first
        if let Some(hash) = extract_hash_from_filename(filename) {
            // Determine media type from extension and insert hash
            match path.extension().and_then(|e| e.to_str()) {
                Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("webp") => {
                    state.recent_photo_hashes.insert(hash);
                }
                Some("mp4") | Some("webm") | Some("mov") => {
                    state.recent_video_hashes.insert(hash);
                }
                Some("mp3") | Some("m4a") | Some("ogg") | Some("wav") => {
                    state.recent_audio_hashes.insert(hash);
                }
                _ => {}
            }
            continue;
        }

        // Extract media ID from filename if present
        // Pattern: {timestamp}_{id|preview_id}_{media_id}.{ext}
        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() >= 3 {
            // Try to find the media ID (should be the last numeric part before extension)
            for part in parts.iter().rev() {
                if let Some(dot_pos) = part.find('.') {
                    let potential_id = &part[..dot_pos];
                    if potential_id.chars().all(|c| c.is_ascii_digit()) && potential_id.len() > 5 {
                        // Determine media type from extension
                        let ext = &part[dot_pos + 1..];
                        match ext {
                            "jpg" | "jpeg" | "png" | "gif" | "webp" => {
                                state
                                    .recent_photo_media_ids
                                    .insert(potential_id.to_string());
                            }
                            "mp4" | "webm" | "mov" => {
                                state
                                    .recent_video_media_ids
                                    .insert(potential_id.to_string());
                            }
                            "mp3" | "m4a" | "ogg" | "wav" => {
                                state
                                    .recent_audio_media_ids
                                    .insert(potential_id.to_string());
                            }
                            _ => {}
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Check if a file is a duplicate based on its hash.
pub fn is_hash_duplicate(
    path: &Path,
    state: &DownloadState,
    media_type: MediaType,
) -> Result<bool> {
    let hash = hash_file(path, media_type)?;

    let is_dupe = match media_type {
        MediaType::Image => state.is_photo_hash_seen(&hash),
        MediaType::Video => state.is_video_hash_seen(&hash),
        MediaType::Audio => state.is_audio_hash_seen(&hash),
        MediaType::Unknown => false,
    };

    Ok(is_dupe)
}

/// Add a file's hash to the state.
pub fn add_hash_to_state(
    path: &Path,
    state: &mut DownloadState,
    media_type: MediaType,
) -> Result<String> {
    let hash = hash_file(path, media_type)?;

    match media_type {
        MediaType::Image => state.mark_photo_hash_seen(hash.clone()),
        MediaType::Video => state.mark_video_hash_seen(hash.clone()),
        MediaType::Audio => state.mark_audio_hash_seen(hash.clone()),
        MediaType::Unknown => {}
    }

    Ok(hash)
}
