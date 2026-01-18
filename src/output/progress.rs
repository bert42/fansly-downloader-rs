//! Progress bar utilities.

use indicatif::{ProgressBar, ProgressStyle};

/// Create a spinner for long-running operations.
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

/// Create a progress bar for downloads.
pub fn create_download_bar(total: u64) -> ProgressBar {
    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    bar
}

/// Create a progress bar for item counts.
pub fn create_item_bar(total: u64, message: &str) -> ProgressBar {
    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{spinner:.green}} {} [{{bar:40.cyan/blue}}] {{pos}}/{{len}}",
                message
            ))
            .unwrap()
            .progress_chars("#>-"),
    );
    bar
}
