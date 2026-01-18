//! Deduplication module.
//!
//! Provides:
//! - File hashing (perceptual for images, MD5 for others)
//! - MP4 box-aware video hashing
//! - Unified deduplication service

pub mod hash;
pub mod service;
pub mod tracker;

pub use hash::{extract_hash_from_filename, hash_file};
pub use service::DedupService;
pub use tracker::{add_hash_to_state, is_hash_duplicate, scan_existing_files};
