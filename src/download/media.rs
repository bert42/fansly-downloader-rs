//! Media file downloading.

use std::path::{Path, PathBuf};

use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::api::FanslyApi;
use crate::config::Config;
use crate::download::m3u8::download_m3u8;
use crate::download::state::DownloadState;
use crate::error::{Error, Result};
use crate::media::{MediaItem, MediaType};

/// Minimum file size to show progress bar (20 MB).
const PROGRESS_THRESHOLD: u64 = 20 * 1024 * 1024;

/// Download a media item to the specified directory.
pub async fn download_media_item(
    api: &FanslyApi,
    config: &Config,
    state: &mut DownloadState,
    item: &MediaItem,
    target_dir: &Path,
) -> Result<Option<PathBuf>> {
    // Check for duplicate by media ID
    let is_duplicate = match item.media_type() {
        MediaType::Image => state.is_photo_id_seen(&item.media_id),
        MediaType::Video => state.is_video_id_seen(&item.media_id),
        MediaType::Audio => state.is_audio_id_seen(&item.media_id),
        MediaType::Unknown => false,
    };

    if is_duplicate {
        state.increment_duplicate();
        if config.options.show_skipped_downloads {
            tracing::debug!("Skipping duplicate media ID: {}", item.media_id);
        }
        return Ok(None);
    }

    // Determine output path
    let filename = item.generate_filename();
    let output_path = target_dir.join(&filename);

    // Check if file already exists
    if output_path.exists() {
        state.increment_duplicate();
        if config.options.show_skipped_downloads {
            tracing::debug!("Skipping existing file: {}", output_path.display());
        }
        return Ok(None);
    }

    // Ensure target directory exists
    tokio::fs::create_dir_all(target_dir).await?;

    // Download the file
    let downloaded_path = if item.is_m3u8() {
        download_m3u8(api, item, &output_path).await?
    } else {
        download_direct(api, config, item, &output_path).await?
    };

    // Mark as seen and update stats
    match item.media_type() {
        MediaType::Image => {
            state.mark_photo_id_seen(item.media_id.clone());
            state.increment_pic();
        }
        MediaType::Video => {
            state.mark_video_id_seen(item.media_id.clone());
            state.increment_vid();
        }
        MediaType::Audio => {
            state.mark_audio_id_seen(item.media_id.clone());
            state.increment_audio();
        }
        MediaType::Unknown => {}
    }

    if config.options.show_downloads {
        tracing::info!("Downloaded: {}", downloaded_path.display());
    }

    Ok(Some(downloaded_path))
}

/// Download a file directly (non-M3U8).
async fn download_direct(
    api: &FanslyApi,
    config: &Config,
    item: &MediaItem,
    output_path: &Path,
) -> Result<PathBuf> {
    let response = api.download_file(&item.download_url).await?;

    let content_length = response.content_length();
    let show_progress = config.options.show_downloads
        && content_length.map(|l| l > PROGRESS_THRESHOLD).unwrap_or(false);

    // Create progress bar if needed
    let progress = if show_progress {
        let pb = ProgressBar::new(content_length.unwrap_or(0));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    // Stream to file
    let mut file = File::create(output_path).await?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| Error::Download(format!("Stream error: {}", e)))?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if let Some(ref pb) = progress {
            pb.set_position(downloaded);
        }
    }

    file.flush().await?;

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    Ok(output_path.to_path_buf())
}
