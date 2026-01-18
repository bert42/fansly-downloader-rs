//! Media parsing utilities.

use std::collections::HashMap;

use crate::api::types::{AccountMedia, MediaDetails};
use crate::media::item::MediaItem;

/// Variant selection result: (url, mimetype, width, height, metadata).
type VariantInfo = (String, String, u32, u32, HashMap<String, String>);

/// Parse an AccountMedia into a MediaItem, selecting the best resolution.
pub fn parse_media_info(media: &AccountMedia, include_previews: bool) -> Option<MediaItem> {
    // Skip if no access and not handling previews
    if !media.access && media.preview.is_none() {
        return None;
    }

    // Determine if we're getting preview or main content
    let (is_preview, media_details) = if media.access {
        // Have access to main content
        (false, media.media.as_ref()?)
    } else if include_previews {
        // Only have preview access
        (true, media.preview.as_ref()?)
    } else {
        return None;
    };

    // Find the best resolution variant
    let (url, mimetype, width, height, metadata) = select_best_variant(media_details)?;

    // Determine file extension from URL
    let extension = extract_extension(&url, &mimetype);

    Some(MediaItem {
        media_id: media.id.clone(),
        created_at: media_details.created_at,
        mimetype,
        download_url: url,
        file_extension: extension,
        resolution: (width as u64) * (height as u64),
        height,
        width,
        is_preview,
        metadata,
    })
}

/// Select the best resolution variant from media details.
fn select_best_variant(details: &MediaDetails) -> Option<VariantInfo> {
    let mut best_url = None;
    let mut best_mimetype = details.mimetype.clone();
    let mut best_width = details.width.unwrap_or(0);
    let mut best_height = details.height.unwrap_or(0);
    let mut best_resolution = (best_width as u64) * (best_height as u64);
    let mut best_metadata = HashMap::new();

    // Check default location
    if let Some(loc) = details.locations.first() {
        best_url = Some(loc.location.clone());
        best_metadata = loc.metadata.clone();
    }

    // Check variants for higher resolution
    for variant in &details.variants {
        let variant_width = variant.width.unwrap_or(0);
        let variant_height = variant.height.unwrap_or(0);
        let variant_resolution = (variant_width as u64) * (variant_height as u64);

        // Only consider variants with same base MIME type
        if !is_compatible_mimetype(&best_mimetype, &variant.mimetype) {
            continue;
        }

        if variant_resolution > best_resolution {
            if let Some(loc) = variant.locations.first() {
                best_url = Some(loc.location.clone());
                best_metadata = loc.metadata.clone();
                best_mimetype = variant.mimetype.clone();
                best_width = variant_width;
                best_height = variant_height;
                best_resolution = variant_resolution;
            }
        }
    }

    best_url.map(|url| (url, best_mimetype, best_width, best_height, best_metadata))
}

/// Check if two MIME types are compatible (same base type).
fn is_compatible_mimetype(base: &str, variant: &str) -> bool {
    let base_type = base.split('/').next().unwrap_or("");
    let variant_type = variant.split('/').next().unwrap_or("");

    base_type == variant_type
}

/// Extract file extension from URL and MIME type.
fn extract_extension(url: &str, mimetype: &str) -> String {
    // First try to get from URL
    if let Some(ext) = extract_extension_from_url(url) {
        return ext;
    }

    // Fall back to MIME type
    mime_to_extension(mimetype)
}

/// Extract extension from URL path.
fn extract_extension_from_url(url: &str) -> Option<String> {
    // Remove query string
    let path = url.split('?').next()?;

    // Get the last segment
    let filename = path.rsplit('/').next()?;

    // Get extension
    let ext = filename.rsplit('.').next()?;

    // Validate it looks like an extension (1-10 chars, alphanumeric)
    if !ext.is_empty() && ext.len() <= 10 && ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        Some(ext.to_lowercase())
    } else {
        None
    }
}

/// Convert MIME type to file extension.
fn mime_to_extension(mimetype: &str) -> String {
    match mimetype {
        // Images
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",

        // Videos
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        "application/vnd.apple.mpegurl" => "mp4", // M3U8 → mp4 after processing

        // Audio
        "audio/mpeg" => "mp3",
        "audio/mp4" => "m4a",
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",

        // Default
        _ => "bin",
    }
    .to_string()
}

/// Extract unique media IDs from timeline/messages response.
pub fn extract_media_ids(
    account_media: &[AccountMedia],
    account_media_bundles: &[crate::api::types::MediaBundle],
) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();

    // Direct media
    for media in account_media {
        ids.push(media.id.clone());
    }

    // Bundle media
    for bundle in account_media_bundles {
        ids.extend(bundle.account_media_ids.clone());
    }

    // Remove duplicates while preserving order
    let mut seen = std::collections::HashSet::new();
    ids.retain(|id| seen.insert(id.clone()));

    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_extension_from_url() {
        assert_eq!(
            extract_extension_from_url("https://example.com/file.jpg"),
            Some("jpg".to_string())
        );
        assert_eq!(
            extract_extension_from_url("https://example.com/file.jpg?token=abc"),
            Some("jpg".to_string())
        );
        assert_eq!(
            extract_extension_from_url("https://example.com/path/to/file.PNG"),
            Some("png".to_string())
        );
    }

    #[test]
    fn test_mime_to_extension() {
        assert_eq!(mime_to_extension("image/jpeg"), "jpg");
        assert_eq!(mime_to_extension("video/mp4"), "mp4");
        assert_eq!(mime_to_extension("audio/mpeg"), "mp3");
        assert_eq!(mime_to_extension("unknown/type"), "bin");
    }
}
