//! Output module for console output and progress.
//!
//! Provides:
//! - Colored console output
//! - Progress bars
//! - Statistics reporting

pub mod console;
pub mod progress;
pub mod stats;

pub use console::{
    print_banner, print_config_summary, print_debug, print_error, print_info, print_success,
    print_warning,
};
pub use progress::{create_download_bar, create_item_bar, create_spinner};
pub use stats::{print_creator_stats, print_global_stats, print_summary};
