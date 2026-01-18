//! Filesystem module.
//!
//! Provides:
//! - Path and directory management
//! - Filename generation and manipulation

pub mod naming;
pub mod paths;

pub use naming::{
    has_hash_in_filename, inject_hash_into_filename, make_unique_filename, sanitize_filename,
};
pub use paths::{ensure_dir, get_creator_folder, get_download_path};
