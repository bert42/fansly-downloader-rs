//! Single post download logic.

use std::time::Duration;

use rand::Rng;
use tokio::time::sleep;

use crate::api::FanslyApi;
use crate::config::Config;
use crate::download::media::download_media_item;
use crate::download::state::DownloadState;
use crate::error::{Error, Result};
use crate::fs::paths::get_download_path;
use crate::media::{extract_media_ids, parse_media_info};

/// Download a single post by ID.
pub async fn download_single_post(
    api: &FanslyApi,
    config: &Config,
    state: &mut DownloadState,
    post_id: &str,
) -> Result<()> {
    tracing::info!("Downloading single post: {}", post_id);

    // Fetch the post
    let post_response = api.get_post(post_id).await?;

    if post_response.posts.is_empty() {
        return Err(Error::Api(format!("Post not found: {}", post_id)));
    }

    // Extract media IDs
    let media_ids = extract_media_ids(
        &post_response.account_media,
        &post_response.account_media_bundles,
    );

    if media_ids.is_empty() {
        tracing::info!("No media found in post {}", post_id);
        return Ok(());
    }

    tracing::info!("Found {} media items in post", media_ids.len());

    // Fetch and download media
    let media_infos = api.get_media_info(&media_ids).await?;

    for media_info in &media_infos {
        if let Some(item) = parse_media_info(media_info, config.options.download_media_previews) {
            let target_dir = get_download_path(config, state, &item)?;

            // Rate limiting delay between downloads
            let delay_ms = rand::thread_rng().gen_range(400..750);
            sleep(Duration::from_millis(delay_ms)).await;

            if let Err(e) = download_media_item(api, config, state, &item, &target_dir).await {
                tracing::warn!("Failed to download media {}: {}", item.media_id, e);
            }
        }
    }

    tracing::info!(
        "Single post download complete: {} pictures, {} videos",
        state.pic_count,
        state.vid_count
    );

    Ok(())
}
