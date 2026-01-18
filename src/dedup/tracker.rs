//! Deduplication tracking.
//!
//! This module provides compatibility functions that delegate to the DedupService.

use std::path::Path;

use crate::download::DownloadState;
use crate::error::Result;
use crate::media::MediaType;

/// Scan a directory for existing files and populate the download state with their hashes.
pub fn scan_existing_files(dir: &Path, state: &mut DownloadState) -> Result<()> {
    state.dedup.scan_directory(dir)
}

/// Check if a file is a duplicate based on its hash.
pub fn is_hash_duplicate(
    path: &Path,
    state: &DownloadState,
    media_type: MediaType,
) -> Result<bool> {
    state.dedup.is_file_duplicate(path, media_type)
}

/// Add a file's hash to the state.
pub fn add_hash_to_state(
    path: &Path,
    state: &mut DownloadState,
    media_type: MediaType,
) -> Result<String> {
    state.dedup.add_file_hash(path, media_type)
}
