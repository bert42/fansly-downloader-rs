//! Filename generation and manipulation.

use std::path::Path;

use crate::error::{Error, Result};

/// Validate and sanitize a filename by removing or replacing invalid characters.
///
/// Returns an error if the filename contains path traversal patterns.
pub fn sanitize_filename(name: &str) -> Result<String> {
    // Reject path traversal attempts
    if name.contains("..") {
        return Err(Error::InvalidFilename(format!(
            "Path traversal detected: '{}'",
            name
        )));
    }

    // Also reject if it contains path separators (should be sanitized, not allowed)
    if name.contains('/') || name.contains('\\') {
        return Err(Error::InvalidFilename(format!(
            "Path separators not allowed in filename: '{}'",
            name
        )));
    }

    // Reject null bytes
    if name.contains('\0') {
        return Err(Error::InvalidFilename(format!(
            "Null bytes not allowed in filename: '{}'",
            name
        )));
    }

    // Sanitize remaining problematic characters
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();

    // Reject empty or whitespace-only names
    if sanitized.trim().is_empty() {
        return Err(Error::InvalidFilename(
            "Filename cannot be empty or whitespace-only".to_string(),
        ));
    }

    Ok(sanitized)
}

/// Sanitize a path component (folder or file name) with less strict validation.
///
/// This is used for creator names and other path components where we want to
/// sanitize rather than reject on certain characters.
pub fn sanitize_path_component(name: &str) -> Result<String> {
    // Reject path traversal attempts
    if name.contains("..") {
        return Err(Error::InvalidFilename(format!(
            "Path traversal detected: '{}'",
            name
        )));
    }

    // Reject null bytes
    if name.contains('\0') {
        return Err(Error::InvalidFilename(format!(
            "Null bytes not allowed: '{}'",
            name
        )));
    }

    // Sanitize problematic characters (replace with underscore)
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();

    // Reject empty or whitespace-only names
    if sanitized.trim().is_empty() {
        return Err(Error::InvalidFilename(
            "Path component cannot be empty or whitespace-only".to_string(),
        ));
    }

    Ok(sanitized)
}

/// Inject a hash into a filename.
///
/// Given a filename like "2024-01-01_id_123.jpg", produces "2024-01-01_id_123_hash2_HASH.jpg"
pub fn inject_hash_into_filename(filename: &str, hash: &str) -> String {
    if let Some(dot_pos) = filename.rfind('.') {
        let name = &filename[..dot_pos];
        let ext = &filename[dot_pos..];
        format!("{}_hash2_{}{}", name, hash, ext)
    } else {
        format!("{}_hash2_{}", filename, hash)
    }
}

/// Check if a filename already contains a hash.
pub fn has_hash_in_filename(filename: &str) -> bool {
    filename.contains("_hash2_") || filename.contains("_hash1_") || filename.contains("_hash_")
}

/// Generate a unique filename by appending a number if the file exists.
pub fn make_unique_filename(path: &Path) -> std::path::PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut counter = 1;
    loop {
        let new_name = if ext.is_empty() {
            format!("{}_{}", stem, counter)
        } else {
            format!("{}_{}.{}", stem, counter, ext)
        };

        let new_path = parent.join(&new_name);
        if !new_path.exists() {
            return new_path;
        }

        counter += 1;
        if counter > 1000 {
            // Safety limit
            return new_path;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_valid() {
        assert_eq!(sanitize_filename("normal.txt").unwrap(), "normal.txt");
        assert_eq!(sanitize_filename("file:name.txt").unwrap(), "file_name.txt");
        assert_eq!(
            sanitize_filename("file*with?special.txt").unwrap(),
            "file_with_special.txt"
        );
    }

    #[test]
    fn test_sanitize_filename_path_traversal() {
        assert!(sanitize_filename("../etc/passwd").is_err());
        assert!(sanitize_filename("..\\windows\\system32").is_err());
        assert!(sanitize_filename("foo/../bar").is_err());
    }

    #[test]
    fn test_sanitize_filename_path_separators() {
        assert!(sanitize_filename("path/to/file.txt").is_err());
        assert!(sanitize_filename("path\\to\\file.txt").is_err());
    }

    #[test]
    fn test_sanitize_filename_null_bytes() {
        assert!(sanitize_filename("file\0name.txt").is_err());
    }

    #[test]
    fn test_sanitize_filename_empty() {
        assert!(sanitize_filename("").is_err());
        assert!(sanitize_filename("   ").is_err());
    }

    #[test]
    fn test_sanitize_path_component_valid() {
        assert_eq!(
            sanitize_path_component("creator_name").unwrap(),
            "creator_name"
        );
        // Path separators are sanitized (not rejected) in path components
        assert_eq!(
            sanitize_path_component("path/to/name").unwrap(),
            "path_to_name"
        );
    }

    #[test]
    fn test_sanitize_path_component_traversal() {
        assert!(sanitize_path_component("../evil").is_err());
        assert!(sanitize_path_component("foo/../bar").is_err());
    }

    #[test]
    fn test_inject_hash() {
        assert_eq!(
            inject_hash_into_filename("file.jpg", "abc123"),
            "file_hash2_abc123.jpg"
        );
        assert_eq!(
            inject_hash_into_filename("no_extension", "abc123"),
            "no_extension_hash2_abc123"
        );
    }

    #[test]
    fn test_has_hash_in_filename() {
        assert!(has_hash_in_filename("file_hash2_abc.jpg"));
        assert!(has_hash_in_filename("file_hash1_abc.jpg"));
        assert!(has_hash_in_filename("file_hash_abc.jpg"));
        assert!(!has_hash_in_filename("file.jpg"));
    }
}
