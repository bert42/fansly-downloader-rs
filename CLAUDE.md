# CLAUDE.md - Project Guide for AI Assistants

## Project Overview

Fansly Downloader RS is a Rust CLI application that downloads media content from Fansly creators. It's a reimplementation of the Python fansly-downloader-ng project.

## Project Structure

```
fansly-downloader-rs/
├── Cargo.toml              # Dependencies and project metadata
├── config.example.toml     # Example configuration file
├── src/
│   ├── main.rs             # Entry point, CLI setup, creator processing loop
│   ├── lib.rs              # Library root, public exports
│   ├── error.rs            # Error types (thiserror), exit codes
│   ├── cli/
│   │   ├── mod.rs
│   │   └── args.rs         # Clap CLI argument definitions
│   ├── config/
│   │   ├── mod.rs
│   │   ├── config.rs       # Config struct, TOML parsing, serde
│   │   ├── validation.rs   # Username/config validation
│   │   └── modes.rs        # DownloadMode, DownloadType enums
│   ├── api/
│   │   ├── mod.rs
│   │   ├── client.rs       # FanslyApi HTTP client (reqwest)
│   │   ├── websocket.rs    # WebSocket session ID retrieval
│   │   ├── auth.rs         # cyrb53 hash, device ID management
│   │   └── types.rs        # API response types (serde deserialize)
│   ├── download/
│   │   ├── mod.rs
│   │   ├── state.rs        # DownloadState, GlobalState tracking
│   │   ├── timeline.rs     # Timeline download with cursor pagination
│   │   ├── messages.rs     # Direct messages download
│   │   ├── single.rs       # Single post download
│   │   ├── collections.rs  # Purchased collections download
│   │   ├── media.rs        # File download orchestration
│   │   └── m3u8.rs         # HLS playlist parsing, ffmpeg concat
│   ├── media/
│   │   ├── mod.rs
│   │   ├── item.rs         # MediaItem struct
│   │   └── parser.rs       # Parse API responses → MediaItem
│   ├── dedup/
│   │   ├── mod.rs
│   │   ├── hash.rs         # Perceptual (images), MD5 (video/audio)
│   │   └── tracker.rs      # Scan existing files, track seen hashes
│   ├── fs/
│   │   ├── mod.rs
│   │   ├── paths.rs        # Directory structure, creator folders
│   │   └── naming.rs       # Filename generation with timestamps
│   └── output/
│       ├── mod.rs
│       ├── console.rs      # Colored output (console crate)
│       ├── progress.rs     # Progress bars (indicatif)
│       └── stats.rs        # Statistics reporting
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `reqwest` | HTTP client with streaming |
| `tokio-tungstenite` | WebSocket for session ID |
| `clap` | CLI argument parsing with derive |
| `serde` + `toml` | Config file parsing |
| `tracing` + `tracing-subscriber` | Logging |
| `indicatif` | Progress bars |
| `image_hasher` | Perceptual image hashing |
| `md-5` | Video/audio file hashing |
| `m3u8-rs` | HLS playlist parsing |
| `thiserror` | Error type definitions |
| `chrono` | Timestamp formatting |
| `console` | Terminal colors and styling |

## Architecture Notes

### Authentication Flow
1. WebSocket connects to `wss://wsv3.fansly.com` to get session ID
2. Device ID generated as UUID, rotated every 180 minutes
3. Each request includes headers: `authorization`, `fansly-client-id`, `fansly-client-ts`, `fansly-client-check`, `fansly-session-id`
4. `fansly-client-check` is generated using cyrb53 hash of `{device_id}:{timestamp}:{check_key}`

### Download Flow
1. `main.rs` loads config, initializes API client, iterates creators
2. `process_creator()` fetches creator info, calls appropriate download function
3. Download functions use cursor-based pagination
4. Media items are parsed, deduplicated, then downloaded via `download_media_item()`
5. M3U8 videos: parse playlist → concurrent segment download → ffmpeg concat

### Deduplication Strategy
- **Media IDs**: Tracked in HashSet per session (recent_*_media_ids)
- **Content Hashes**: Perceptual hash for images, MD5 for video/audio
- **Filename Scanning**: `scan_existing_files()` extracts hashes from existing filenames

### Error Handling
- Custom `Error` enum in `error.rs` using thiserror
- `Result<T>` type alias throughout
- Exit codes defined for different error categories (config, API, download)

## Common Tasks

### Adding a New Download Mode
1. Add variant to `DownloadMode` enum in `config/modes.rs`
2. Create new function in `download/` module
3. Add match arm in `process_creator()` in `main.rs`
4. Update CLI args in `cli/args.rs`

### Adding New API Endpoint
1. Add response types to `api/types.rs`
2. Add method to `FanslyApi` in `api/client.rs`
3. Use `self.get()` or `self.post()` with proper endpoint

### Modifying File Naming
- Edit `generate_filename()` in `fs/naming.rs`
- Pattern: `{timestamp}_{post_id}_{media_id}[_{hash}].{ext}`

## Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Testing

To test the application:
1. Create `config.toml` with valid credentials
2. Run against a creator with known content
3. Verify file downloads, naming, and deduplication
4. Test each download mode separately
5. Test M3U8 videos (requires ffmpeg in PATH)

## Known Limitations

- M3U8/HLS requires ffmpeg installed
- No GUI, CLI only
- Single-threaded download per creator (concurrent segments for M3U8)
- In-memory deduplication (hashes not persisted between runs unless in filenames)
