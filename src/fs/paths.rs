//! Path and directory management.

use std::path::PathBuf;

use crate::config::{Config, DownloadType};
use crate::download::DownloadState;
use crate::error::Result;
use crate::media::MediaItem;

/// Get the download path for a media item.
pub fn get_download_path(
    config: &Config,
    state: &DownloadState,
    item: &MediaItem,
) -> Result<PathBuf> {
    let base_dir = config.download_directory();

    // Build creator folder name
    let creator_name = state.creator_name.as_deref().unwrap_or("unknown_creator");

    let creator_folder = if config.options.use_folder_suffix {
        format!("{}_fansly", creator_name)
    } else {
        creator_name.to_string()
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
pub fn get_creator_folder(config: &Config, creator_name: &str) -> PathBuf {
    let base_dir = config.download_directory();

    let creator_folder = if config.options.use_folder_suffix {
        format!("{}_fansly", creator_name)
    } else {
        creator_name.to_string()
    };

    base_dir.join(&creator_folder)
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

        let path = get_creator_folder(&config, "testuser");
        assert_eq!(path, PathBuf::from("/downloads/testuser_fansly"));

        config.options.use_folder_suffix = false;
        let path = get_creator_folder(&config, "testuser");
        assert_eq!(path, PathBuf::from("/downloads/testuser"));
    }
}
