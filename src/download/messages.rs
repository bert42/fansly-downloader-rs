//! Messages download logic.

use std::time::Duration;

use rand::Rng;
use tokio::time::sleep;

use crate::api::{FanslyApi, BATCH_SIZE};
use crate::config::Config;
use crate::download::media::download_media_item;
use crate::download::state::DownloadState;
use crate::error::Result;
use crate::fs::paths::get_download_path;
use crate::media::{extract_media_ids, parse_media_info};

/// Default duplicate threshold percentage for messages.
const DUPLICATE_THRESHOLD_PERCENT: f64 = 0.2;

/// Download messages for a creator.
pub async fn download_messages(
    api: &FanslyApi,
    config: &Config,
    state: &mut DownloadState,
) -> Result<()> {
    let creator_id = state.creator_id()?.to_string();

    tracing::info!(
        "Downloading messages for {}...",
        state.creator_name.as_deref().unwrap_or("unknown")
    );

    // Find the message group with this creator
    let groups = api.get_groups().await?;
    let group = groups
        .iter()
        .find(|g| g.users.iter().any(|u| u.user_id == creator_id));

    let group = match group {
        Some(g) => g,
        None => {
            tracing::info!(
                "No chat history with {}",
                state.creator_name.as_deref().unwrap_or("this creator")
            );
            return Ok(());
        }
    };

    let group_id = group.id.clone();
    let mut cursor = "0".to_string();
    let mut total_items = 0u64;

    loop {
        // Rate limiting delay between pages
        let delay_ms = rand::thread_rng().gen_range(2000..4000);
        sleep(Duration::from_millis(delay_ms)).await;

        // Fetch messages page
        let messages = api.get_messages(&group_id, &cursor).await?;

        // Extract media IDs
        let media_ids = extract_media_ids(&messages.account_media, &messages.account_media_bundles);

        if media_ids.is_empty() && messages.messages.is_empty() {
            tracing::debug!("No more messages");
            break;
        }

        total_items += media_ids.len() as u64;

        // Fetch and download media in batches
        for chunk in media_ids.chunks(BATCH_SIZE) {
            // Rate limiting delay between batches
            let delay_ms = rand::thread_rng().gen_range(400..750);
            sleep(Duration::from_millis(delay_ms)).await;

            let media_infos = api.get_media_info(chunk).await?;

            for media_info in &media_infos {
                if let Some(item) =
                    parse_media_info(media_info, config.options.download_media_previews)
                {
                    let target_dir = get_download_path(config, state, &item)?;

                    // Rate limiting delay between downloads
                    let delay_ms = rand::thread_rng().gen_range(400..750);
                    sleep(Duration::from_millis(delay_ms)).await;

                    if let Err(e) =
                        download_media_item(api, config, state, &item, &target_dir).await
                    {
                        tracing::warn!("Failed to download media {}: {}", item.media_id, e);
                    }
                }
            }
        }

        // Check duplicate threshold
        if config.options.use_duplicate_threshold {
            let threshold = (total_items as f64 * DUPLICATE_THRESHOLD_PERCENT) as u64;
            if state.duplicate_count > threshold.max(50) {
                tracing::info!(
                    "Duplicate threshold reached ({} duplicates), stopping messages download",
                    state.duplicate_count
                );
                break;
            }
        }

        // Get next cursor from last message
        cursor = messages
            .messages
            .last()
            .map(|m| m.id.clone())
            .unwrap_or_else(|| "0".to_string());

        if messages.messages.is_empty() {
            break;
        }
    }

    tracing::info!(
        "Messages download complete: {} pictures, {} videos",
        state.pic_count,
        state.vid_count
    );

    Ok(())
}
