//! Fansly Downloader RS - CLI entry point.

use std::process::ExitCode;

use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

use fansly_downloader::{
    api::FanslyApi,
    cli::Args,
    config::{parse_post_id, validate_config, Config, DownloadMode, DownloadType},
    download::{
        download_collections, download_messages, download_single_post, download_timeline,
        DownloadState, GlobalState,
    },
    error::{exit_codes, Error, Result},
    fs::get_creator_folder,
    output::{
        print_banner, print_config_summary, print_creator_stats, print_error, print_global_stats,
        print_info, print_warning,
    },
};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::from(exit_codes::SUCCESS as u8),
        Err(e) => {
            print_error(&format!("{}", e));
            match e {
                Error::Config(_) | Error::ConfigValidation { .. } | Error::MissingConfig(_) => {
                    ExitCode::from(exit_codes::CONFIG_ERROR as u8)
                }
                Error::Authentication(_) | Error::Api(_) | Error::AccountNotFound(_) => {
                    ExitCode::from(exit_codes::API_ERROR as u8)
                }
                Error::Download(_) | Error::M3U8(_) => {
                    ExitCode::from(exit_codes::DOWNLOAD_ERROR as u8)
                }
                _ => ExitCode::from(exit_codes::UNEXPECTED_ERROR as u8),
            }
        }
    }
}

async fn run() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Set up logging
    let log_level = if args.debug { "debug" } else { "info" };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    fmt().with_env_filter(filter).with_target(false).init();

    // Print banner
    print_banner();

    // Load configuration
    let config_path = args.config.clone();
    let mut config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        print_warning(&format!(
            "Configuration file not found: {}",
            config_path.display()
        ));
        print_info("Using default configuration with CLI arguments");
        Config {
            targeted_creator: Default::default(),
            my_account: Default::default(),
            options: Default::default(),
            cache: Default::default(),
        }
    };

    // Merge CLI arguments into config
    args.merge_into_config(&mut config);

    // Validate configuration
    validate_config(&config)?;

    // Print configuration summary
    let creators: Vec<String> = config.targeted_creator.usernames.iter().cloned().collect();
    print_config_summary(
        &creators,
        &config.options.download_mode.to_string(),
        &config.download_directory().display().to_string(),
    );

    // Initialize API client
    print_info("Connecting to Fansly...");
    let api = FanslyApi::new(
        config.my_account.authorization_token.clone(),
        config.my_account.user_agent.clone(),
        config.my_account.check_key.clone(),
        config.cache.device_id.clone(),
        config.cache.device_id_timestamp,
    )
    .await?;

    // Validate token by fetching account info
    let account_info = api.get_client_account_info().await?;
    print_info(&format!(
        "Logged in as: {}",
        account_info
            .display_name
            .as_deref()
            .unwrap_or(&account_info.username)
    ));

    // Update cached device ID
    let device_id = api.get_device_id().await?;
    let device_id_timestamp = api.get_device_id_timestamp().await;
    if let Some(timestamp) = device_id_timestamp {
        config.update_cache(device_id, timestamp, Some(&config_path))?;
    }

    // Initialize global state
    let mut global_state = GlobalState::default();

    // Process each creator
    for creator_name in &creators {
        print_info(&format!("Processing creator: {}", creator_name));

        match process_creator(&api, &config, creator_name).await {
            Ok(state) => {
                print_creator_stats(&state);
                global_state.add_creator_stats(&state);
            }
            Err(e) => {
                print_error(&format!("Failed to process {}: {}", creator_name, e));
                global_state.mark_creator_failed();
            }
        }
    }

    // Print global statistics
    print_global_stats(&global_state);

    if global_state.creators_failed > 0 {
        return Err(Error::Api(format!(
            "{} creator(s) failed",
            global_state.creators_failed
        )));
    }

    Ok(())
}

/// Process a single creator.
async fn process_creator(
    api: &FanslyApi,
    config: &Config,
    creator_name: &str,
) -> Result<DownloadState> {
    // Get creator account info
    let creator_info = api.get_creator_account_info(creator_name).await?;

    // Initialize download state
    let mut state = DownloadState::new(creator_name.to_string(), creator_info.id.clone());
    state.following = creator_info.following.unwrap_or(false);
    state.subscribed = creator_info.subscribed.unwrap_or(false);

    // Set base path (with path traversal protection)
    state.base_path = Some(get_creator_folder(config, creator_name)?);

    // Execute based on download mode
    match config.options.download_mode {
        DownloadMode::Normal => {
            // Download both timeline and messages
            state.download_type = DownloadType::Timeline;
            download_timeline(api, config, &mut state).await?;

            state.download_type = DownloadType::Messages;
            if let Err(e) = download_messages(api, config, &mut state).await {
                print_warning(&format!("Messages download failed: {}", e));
            }
        }
        DownloadMode::Timeline => {
            state.download_type = DownloadType::Timeline;
            download_timeline(api, config, &mut state).await?;
        }
        DownloadMode::Messages => {
            state.download_type = DownloadType::Messages;
            download_messages(api, config, &mut state).await?;
        }
        DownloadMode::Single => {
            state.download_type = DownloadType::Single;
            let post_id = config
                .options
                .single_post_id
                .as_ref()
                .ok_or_else(|| Error::Config("Post ID required for single mode".into()))?;
            let post_id = parse_post_id(post_id)?;
            download_single_post(api, config, &mut state, &post_id).await?;
        }
        DownloadMode::Collection => {
            state.download_type = DownloadType::Collections;
            download_collections(api, config, &mut state).await?;
        }
    }

    Ok(state)
}
