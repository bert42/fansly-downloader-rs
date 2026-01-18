# Fansly Downloader RS

A Rust CLI tool to download photos, videos, and audio from Fansly creators.

This is a Rust reimplementation of [fansly-downloader-ng](https://github.com/prof79/fansly-downloader-ng), offering improved performance and a single binary distribution.

## Features

- **Multiple Download Modes**: Timeline posts, direct messages, single posts, and purchased collections
- **Smart Deduplication**: Perceptual hashing for images, MD5 for video/audio to avoid duplicate downloads
- **HLS/M3U8 Support**: Automatically downloads and concatenates video streams using ffmpeg
- **Progress Tracking**: Real-time progress bars and download statistics
- **Flexible Configuration**: TOML config file with CLI argument overrides
- **Rate Limiting**: Built-in delays to respect API limits

## Requirements

- **ffmpeg**: Required for M3U8/HLS video downloads (must be in PATH)

## Installation

### From Source

```bash
git clone https://github.com/yourusername/fansly-downloader-rs.git
cd fansly-downloader-rs
cargo build --release
```

The binary will be at `./target/release/fansly-downloader`.

## Configuration

### Getting Your Credentials

You'll need two pieces of information from your browser:

1. **Authorization Token** (required): Your Fansly session token
2. **Device ID** (required): Your browser's device ID from the `fansly-d` cookie
3. **User Agent** (optional): Your browser's user agent string - a default is provided
4. **Check Key** (optional): Fansly's request signing key - a default is provided

### How to Get These Values

1. Open **Developer Tools** (F12) while on Fansly
2. Go to the **Network** tab
3. Filter for `apiv3` and click any request
4. Find in **Request Headers**:
   - `authorization` → your token
   - `Fansly-Client-Id` → your device ID
5. Or go to **Application** tab → **Cookies** → look for `fansly-d` cookie value

### Config File (Recommended)

Create a `config.toml` file:

```toml
[my_account]
authorization_token = "your_token_here"
# user_agent and check_key are optional (sensible defaults provided)

[targeted_creator]
usernames = ["creator1", "creator2"]

[cache]
device_id = "your_device_id_here"

[options]
download_directory = "./downloads"
download_mode = "normal"  # normal, timeline, messages, single, collection
use_folder_suffix = true
download_previews = true
show_downloads = true
show_skipped_downloads = false
use_duplicate_threshold = false
timeline_retries = 1
timeline_retry_delay = 60
```

### CLI Arguments

All config options can be overridden via CLI:

```bash
fansly-downloader \
  --user creator_name \
  --token YOUR_TOKEN \
  --directory ./downloads \
  --mode timeline
```

Environment variables are also supported:
- `FANSLY_TOKEN`
- `FANSLY_USER_AGENT`
- `FANSLY_CHECK_KEY`

## Usage

### Download Timeline and Messages (Default)

```bash
fansly-downloader -c config.toml
```

### Download Only Timeline

```bash
fansly-downloader --mode timeline --user creator_name
```

### Download Only Messages

```bash
fansly-downloader --mode messages --user creator_name
```

### Download Single Post

```bash
fansly-downloader --mode single --post 123456789 --user creator_name
```

### Download Purchased Collections

```bash
fansly-downloader --mode collection --user creator_name
```

### Multiple Creators

```bash
fansly-downloader --user creator1 creator2 creator3
```

## CLI Options

| Option | Description |
|--------|-------------|
| `-u, --user <USER>...` | Creator username(s) to download |
| `-d, --directory <PATH>` | Download directory |
| `-t, --token <TOKEN>` | Authorization token |
| `-a, --user-agent <UA>` | Browser user agent |
| `-k, --check-key <KEY>` | Fansly check key |
| `-c, --config <FILE>` | Config file path (default: config.toml) |
| `--mode <MODE>` | Download mode: normal, timeline, messages, single, collection |
| `--post <ID>` | Post ID for single mode |
| `--no-folder-suffix` | Don't add "_fansly" suffix to folders |
| `--no-previews` | Skip preview media |
| `-q, --quiet` | Hide progress bars |
| `--show-skipped` | Show skipped download info |
| `--use-duplicate-threshold` | Stop after too many duplicates |
| `--timeline-retries <N>` | Retry attempts for empty timelines |
| `--timeline-delay <SECS>` | Delay between retries |
| `--debug` | Enable debug logging |

## File Organization

Downloads are organized as:

```
download_dir/
└── CreatorName_fansly/
    ├── Pictures/
    │   └── Previews/  (if separate_previews enabled)
    ├── Videos/
    │   └── Previews/
    ├── Audio/
    ├── Timeline/      (if separate_timeline enabled)
    └── Messages/      (if separate_messages enabled)
```

Filename format: `{timestamp}_{post_id}_{media_id}.{ext}`

## License

MIT License

## Acknowledgments

- Inspired by [fansly-downloader-ng](https://github.com/prof79/fansly-downloader-ng) by prof79
