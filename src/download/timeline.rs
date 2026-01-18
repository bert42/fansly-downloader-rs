//! Timeline download logic.

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

/// Default duplicate threshold percentage.
const DUPLICATE_THRESHOLD_PERCENT: f64 = 0.2;

/// Download timeline posts for a creator.
pub async fn download_timeline(
    api: &FanslyApi,
    config: &Config,
    state: &mut DownloadState,
) -> Result<()> {
    let creator_id = state.creator_id()?.to_string();
    let mut cursor = "0".to_string();
    let mut empty_response_count = 0;
    let mut total_items = 0u64;

    tracing::info!(
        "Downloading timeline for {}...",
        state.creator_name.as_deref().unwrap_or("unknown")
    );

    loop {
        // Rate limiting delay between pages
        let delay_ms = rand::thread_rng().gen_range(2000..4000);
        sleep(Duration::from_millis(delay_ms)).await;

        // Fetch timeline page
        let timeline = api.get_timeline(&creator_id, &cursor).await?;

        // Extract media IDs
        let media_ids = extract_media_ids(&timeline.account_media, &timeline.account_media_bundles);

        if media_ids.is_empty() {
            empty_response_count += 1;

            if empty_response_count > config.options.timeline_retries {
                tracing::debug!(
                    "No more timeline content after {} retries",
                    empty_response_count - 1
                );
                break;
            }

            tracing::debug!(
                "Empty timeline response, retrying in {} seconds...",
                config.options.timeline_delay_seconds
            );
            sleep(Duration::from_secs(config.options.timeline_delay_seconds)).await;
            continue;
        }

        // Reset retry counter on success
        empty_response_count = 0;
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
            if state.duplicate_count() > threshold.max(50) {
                tracing::info!(
                    "Duplicate threshold reached ({} duplicates), stopping timeline download",
                    state.duplicate_count()
                );
                break;
            }
        }

        // Get next cursor from last post
        cursor = timeline
            .posts
            .last()
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "0".to_string());

        if timeline.posts.is_empty() {
            break;
        }
    }

    tracing::info!(
        "Timeline download complete: {} pictures, {} videos",
        state.pic_count,
        state.vid_count
    );

    Ok(())
}
