//! Collections (purchased media) download logic.

use std::time::Duration;

use rand::Rng;
use tokio::time::sleep;

use crate::api::{FanslyApi, BATCH_SIZE};
use crate::config::Config;
use crate::download::media::download_media_item;
use crate::download::state::DownloadState;
use crate::error::Result;
use crate::fs::paths::get_download_path;
use crate::media::parse_media_info;

/// Download purchased media collections.
pub async fn download_collections(
    api: &FanslyApi,
    config: &Config,
    state: &mut DownloadState,
) -> Result<()> {
    tracing::info!(
        "Downloading collections for {}...",
        state.creator_name.as_deref().unwrap_or("unknown")
    );

    // Fetch all collection orders
    let orders = api.get_collections().await?;

    if orders.is_empty() {
        tracing::info!("No purchased media found in collections");
        return Ok(());
    }

    tracing::info!("Found {} purchased media items", orders.len());

    // Extract media IDs from orders
    let media_ids: Vec<String> = orders.iter().map(|o| o.account_media_id.clone()).collect();

    // Fetch and download media in batches
    for chunk in media_ids.chunks(BATCH_SIZE) {
        // Rate limiting delay between batches
        let delay_ms = rand::thread_rng().gen_range(400..750);
        sleep(Duration::from_millis(delay_ms)).await;

        let media_infos = api.get_media_info(&chunk.to_vec()).await?;

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
    }

    tracing::info!(
        "Collections download complete: {} pictures, {} videos",
        state.pic_count,
        state.vid_count
    );

    Ok(())
}
