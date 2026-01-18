//! Path and directory management.

use std::path::PathBuf;

use crate::config::{Config, DownloadType};
use crate::download::DownloadState;
use crate::error::Result;
use crate::fs::naming::sanitize_path_component;
use crate::media::MediaItem;

/// Get the download path for a media item.
pub fn get_download_path(
    config: &Config,
    state: &DownloadState,
    item: &MediaItem,
) -> Result<PathBuf> {
    let base_dir = config.download_directory();

    // Build creator folder name with sanitization to prevent path traversal
    let creator_name = state.creator_name.as_deref().unwrap_or("unknown_creator");
    let sanitized_name = sanitize_path_component(creator_name)?;

    let creator_folder = if config.options.use_folder_suffix {
        format!("{}_fansly", sanitized_name)
    } else {
        sanitized_name
    };

    let mut path = base_dir.join(&creator_folder);

    // Add download type folder if separated
    match state.download_type {
        DownloadType::Timeline if config.options.separate_timeline => {
            path = path.join("Timeline");
        }
        DownloadType::Messages if config.options.separate_messages => {
            path = path.join("Messages");
        }
        DownloadType::Collections => {
            path = path.join("Collections");
        }
        DownloadType::Single => {
            path = path.join("Single");
        }
        _ => {}
    }

    // Add media type folder
    path = path.join(item.media_type().folder_name());

    // Add previews subfolder if configured
    if item.is_preview && config.options.separate_previews {
        path = path.join("Previews");
    }

    Ok(path)
}

/// Get the base creator folder path.
///
/// Returns an error if the creator name contains path traversal patterns.
pub fn get_creator_folder(config: &Config, creator_name: &str) -> Result<PathBuf> {
    let base_dir = config.download_directory();

    // Sanitize creator name to prevent path traversal
    let sanitized_name = sanitize_path_component(creator_name)?;

    let creator_folder = if config.options.use_folder_suffix {
        format!("{}_fansly", sanitized_name)
    } else {
        sanitized_name
    };

    Ok(base_dir.join(&creator_folder))
}

/// Ensure a directory exists, creating it if necessary.
pub fn ensure_dir(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AccountConfig, CacheConfig, Config, CreatorConfig, OptionsConfig};

    fn make_test_config() -> Config {
        Config {
            targeted_creator: CreatorConfig::default(),
            my_account: AccountConfig::default(),
            options: OptionsConfig::default(),
            cache: CacheConfig::default(),
        }
    }

    #[test]
    fn test_get_creator_folder() {
        let mut config = make_test_config();
        config.options.download_directory = Some(PathBuf::from("/downloads"));
        config.options.use_folder_suffix = true;

        let path = get_creator_folder(&config, "testuser").unwrap();
        assert_eq!(path, PathBuf::from("/downloads/testuser_fansly"));

        config.options.use_folder_suffix = false;
        let path = get_creator_folder(&config, "testuser").unwrap();
        assert_eq!(path, PathBuf::from("/downloads/testuser"));
    }

    #[test]
    fn test_get_creator_folder_path_traversal() {
        let mut config = make_test_config();
        config.options.download_directory = Some(PathBuf::from("/downloads"));

        // Path traversal should be rejected
        assert!(get_creator_folder(&config, "../evil").is_err());
        assert!(get_creator_folder(&config, "foo/../bar").is_err());
    }

    #[test]
    fn test_get_creator_folder_sanitizes_special_chars() {
        let mut config = make_test_config();
        config.options.download_directory = Some(PathBuf::from("/downloads"));
        config.options.use_folder_suffix = false;

        // Special characters should be sanitized
        let path = get_creator_folder(&config, "user/name").unwrap();
        assert_eq!(path, PathBuf::from("/downloads/user_name"));
    }
}
