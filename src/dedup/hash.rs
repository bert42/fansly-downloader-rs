//! File hashing for deduplication.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use image_hasher::{HashAlg, HasherConfig};
use md5::{Digest, Md5};

use crate::error::{Error, Result};
use crate::media::MediaType;

/// Hash size for perceptual image hashing (16 bits = 256 possible values).
const HASH_SIZE: u32 = 16;

/// Compute a hash for a media file based on its type.
pub fn hash_file(path: &Path, media_type: MediaType) -> Result<String> {
    match media_type {
        MediaType::Image => hash_image(path),
        MediaType::Video => hash_video(path),
        MediaType::Audio => hash_md5(path),
        MediaType::Unknown => hash_md5(path),
    }
}

/// Compute perceptual hash for an image file.
fn hash_image(path: &Path) -> Result<String> {
    let image =
        image::open(path).map_err(|e| Error::Media(format!("Failed to open image: {}", e)))?;

    let hasher = HasherConfig::new()
        .hash_size(HASH_SIZE, HASH_SIZE)
        .hash_alg(HashAlg::DoubleGradient)
        .to_hasher();

    let hash = hasher.hash_image(&image);

    // Convert to hex string
    Ok(hash.to_base64())
}

/// Compute hash for a video file (MP4 box-based hashing).
///
/// This excludes the 'moov' and 'free' boxes which contain variable metadata.
fn hash_video(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    let mut hasher = Md5::new();
    let mut position: u64 = 0;

    // Read MP4 boxes and hash non-metadata ones
    while position < file_size {
        // Read box header (8 bytes: 4 for size, 4 for type)
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }

        let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let box_type = String::from_utf8_lossy(&header[4..8]).to_string();

        if box_size == 0 {
            // Box extends to end of file
            let mut remaining = Vec::new();
            reader.read_to_end(&mut remaining)?;
            if !should_skip_box(&box_type) {
                hasher.update(header);
                hasher.update(remaining);
            }
            break;
        }

        if box_size < 8 {
            // Invalid box
            break;
        }

        // Read box content
        let content_size = (box_size - 8) as usize;
        let mut content = vec![0u8; content_size];
        if reader.read_exact(&mut content).is_err() {
            break;
        }

        // Hash the box if it's not a metadata box
        if !should_skip_box(&box_type) {
            hasher.update(header);
            hasher.update(content);
        }

        position += box_size;
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Check if a box type should be skipped during hashing.
fn should_skip_box(box_type: &str) -> bool {
    matches!(box_type, "moov" | "free" | "skip" | "meta" | "udta")
}

/// Compute MD5 hash for a file.
fn hash_md5(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Md5::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Extract hash from a filename if present.
///
/// Looks for patterns like `_hash2_HEXVALUE` in the filename.
pub fn extract_hash_from_filename(filename: &str) -> Option<String> {
    // Find _hash2_ pattern
    if let Some(pos) = filename.find("_hash2_") {
        let after_prefix = &filename[pos + 7..];
        // Find the end (before extension or end of string)
        if let Some(end_pos) = after_prefix.find('.') {
            return Some(after_prefix[..end_pos].to_string());
        } else {
            return Some(after_prefix.to_string());
        }
    }

    // Also check for legacy _hash1_ and _hash_ patterns
    for pattern in ["_hash1_", "_hash_"] {
        if let Some(pos) = filename.find(pattern) {
            let prefix_len = pattern.len();
            let after_prefix = &filename[pos + prefix_len..];
            if let Some(end_pos) = after_prefix.find('.') {
                return Some(after_prefix[..end_pos].to_string());
            } else {
                return Some(after_prefix.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hash_from_filename() {
        assert_eq!(
            extract_hash_from_filename("2024-01-01_id_123_hash2_abc123.jpg"),
            Some("abc123".to_string())
        );
        assert_eq!(extract_hash_from_filename("2024-01-01_id_123.jpg"), None);
    }

    #[test]
    fn test_should_skip_box() {
        assert!(should_skip_box("moov"));
        assert!(should_skip_box("free"));
        assert!(!should_skip_box("mdat"));
        assert!(!should_skip_box("ftyp"));
    }
}
