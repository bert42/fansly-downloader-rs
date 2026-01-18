//! Filename generation and manipulation.

use std::path::Path;

/// Sanitize a filename by removing or replacing invalid characters.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
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
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("file:name.txt"), "file_name.txt");
        assert_eq!(sanitize_filename("path/to/file.txt"), "path_to_file.txt");
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
