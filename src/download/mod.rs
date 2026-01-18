//! Download module for content downloading.
//!
//! This module provides:
//! - Download state tracking
//! - Timeline downloading
//! - Messages downloading
//! - Single post downloading
//! - Collections downloading
//! - Media file downloading
//! - M3U8/HLS handling

pub mod collections;
pub mod m3u8;
pub mod media;
pub mod messages;
pub mod single;
pub mod state;
pub mod timeline;

pub use collections::download_collections;
pub use media::download_media_item;
pub use messages::download_messages;
pub use single::download_single_post;
pub use state::{DownloadState, GlobalState};
pub use timeline::download_timeline;
