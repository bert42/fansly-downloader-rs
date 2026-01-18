//! M3U8/HLS playlist downloading.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use futures::stream::{self, StreamExt};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::api::FanslyApi;
use crate::error::{Error, Result};
use crate::media::MediaItem;

/// Maximum concurrent segment downloads.
const MAX_CONCURRENT_SEGMENTS: usize = 4;

/// Download an M3U8 stream and convert to MP4.
pub async fn download_m3u8(
    api: &FanslyApi,
    item: &MediaItem,
    output_path: &Path,
) -> Result<PathBuf> {
    // Ensure output has .mp4 extension
    let output_path = output_path.with_extension("mp4");

    // Fetch the M3U8 playlist
    let playlist_content = fetch_playlist(api, &item.download_url).await?;

    // Parse the playlist
    let playlist = m3u8_rs::parse_playlist_res(playlist_content.as_bytes())
        .map_err(|e| Error::M3U8(format!("Failed to parse playlist: {:?}", e)))?;

    // Handle master or media playlist
    let segments = match playlist {
        m3u8_rs::Playlist::MasterPlaylist(master) => {
            // Select highest quality variant
            let variant = master
                .variants
                .iter()
                .max_by_key(|v| v.bandwidth)
                .ok_or_else(|| Error::M3U8("No variants in master playlist".into()))?;

            // Build variant URL
            let variant_url = resolve_url(&item.download_url, &variant.uri)?;

            // Fetch the media playlist
            let media_content = fetch_playlist(api, &variant_url).await?;
            let media_playlist = m3u8_rs::parse_playlist_res(media_content.as_bytes())
                .map_err(|e| Error::M3U8(format!("Failed to parse media playlist: {:?}", e)))?;

            match media_playlist {
                m3u8_rs::Playlist::MediaPlaylist(mp) => extract_segments(&variant_url, &mp),
                _ => return Err(Error::M3U8("Expected media playlist".into())),
            }
        }
        m3u8_rs::Playlist::MediaPlaylist(media) => extract_segments(&item.download_url, &media),
    };

    if segments.is_empty() {
        return Err(Error::M3U8("No segments found in playlist".into()));
    }

    // Create temp directory for segments
    let parent = output_path
        .parent()
        .ok_or_else(|| Error::M3U8("Output path has no parent directory".into()))?;
    let temp_dir = parent.join(format!(".m3u8_temp_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).await?;

    // Download segments concurrently
    let segment_paths = download_segments(api, &segments, &temp_dir).await?;

    // Concatenate with ffmpeg
    let result = concatenate_segments(&segment_paths, &output_path).await;

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir).await;

    result?;

    Ok(output_path)
}

/// Fetch playlist content from URL.
async fn fetch_playlist(api: &FanslyApi, url: &str) -> Result<String> {
    let response = api.download_file(url).await?;
    let content = response
        .text()
        .await
        .map_err(|e| Error::M3U8(format!("Failed to read playlist: {}", e)))?;
    Ok(content)
}

/// Extract segment URLs from a media playlist.
fn extract_segments(base_url: &str, playlist: &m3u8_rs::MediaPlaylist) -> Vec<String> {
    playlist
        .segments
        .iter()
        .filter_map(|seg| resolve_url(base_url, &seg.uri).ok())
        .collect()
}

/// Resolve a potentially relative URL against a base URL.
fn resolve_url(base: &str, path: &str) -> Result<String> {
    if path.starts_with("http://") || path.starts_with("https://") {
        return Ok(path.to_string());
    }

    let base_url = url::Url::parse(base)?;
    let resolved = base_url.join(path)?;
    Ok(resolved.to_string())
}

/// Download all segments concurrently.
async fn download_segments(
    api: &FanslyApi,
    segments: &[String],
    temp_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let results: Vec<Result<PathBuf>> = stream::iter(segments.iter().enumerate())
        .map(|(i, url)| {
            let temp_dir = temp_dir.to_path_buf();
            async move {
                let segment_path = temp_dir.join(format!("segment_{:05}.ts", i));
                download_segment(api, url, &segment_path).await?;
                Ok(segment_path)
            }
        })
        .buffer_unordered(MAX_CONCURRENT_SEGMENTS)
        .collect()
        .await;

    // Collect results, preserving order
    let mut paths = Vec::with_capacity(segments.len());
    for result in results {
        paths.push(result?);
    }

    // Sort by filename to ensure correct order
    paths.sort();

    Ok(paths)
}

/// Download a single segment.
async fn download_segment(api: &FanslyApi, url: &str, output: &Path) -> Result<()> {
    let response = api.download_file(url).await?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| Error::M3U8(format!("Failed to download segment: {}", e)))?;

    let mut file = File::create(output).await?;
    file.write_all(&bytes).await?;
    file.flush().await?;

    Ok(())
}

/// Concatenate segments using ffmpeg.
async fn concatenate_segments(segments: &[PathBuf], output: &Path) -> Result<()> {
    // Create concat list file
    let concat_list = output.with_extension("ffc");
    let mut list_content = String::new();

    for segment in segments {
        list_content.push_str(&format!("file '{}'\n", segment.display()));
    }

    fs::write(&concat_list, &list_content).await?;

    // Run ffmpeg
    let concat_list_str = concat_list
        .to_str()
        .ok_or_else(|| Error::M3U8("Invalid path encoding for concat list".into()))?;
    let output_str = output
        .to_str()
        .ok_or_else(|| Error::M3U8("Invalid path encoding for output".into()))?;

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            concat_list_str,
            "-c",
            "copy",
            output_str,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::FFmpegNotFound
            } else {
                Error::FFmpeg(format!("Failed to run ffmpeg: {}", e))
            }
        })?;

    // Clean up concat list
    let _ = fs::remove_file(&concat_list).await;

    if !status.success() {
        return Err(Error::FFmpeg(format!(
            "ffmpeg exited with status: {}",
            status
        )));
    }

    Ok(())
}
