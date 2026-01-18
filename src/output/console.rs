//! Console output utilities.

use console::style;

/// Print an info message.
pub fn print_info(message: &str) {
    println!("{} {}", style("INFO").cyan().bold(), message);
}

/// Print a success message.
pub fn print_success(message: &str) {
    println!("{} {}", style("OK").green().bold(), message);
}

/// Print a warning message.
pub fn print_warning(message: &str) {
    println!("{} {}", style("WARN").yellow().bold(), message);
}

/// Print an error message.
pub fn print_error(message: &str) {
    eprintln!("{} {}", style("ERROR").red().bold(), message);
}

/// Print a debug message.
pub fn print_debug(message: &str) {
    println!("{} {}", style("DEBUG").dim(), message);
}

/// Print the application banner.
pub fn print_banner() {
    let banner = r#"
╔═══════════════════════════════════════════════════════╗
║     Fansly Downloader RS                              ║
║     A Rust implementation of fansly-downloader-ng     ║
╚═══════════════════════════════════════════════════════╝
"#;
    println!("{}", style(banner).cyan());
}

/// Print configuration summary.
pub fn print_config_summary(
    creators: &[String],
    download_mode: &str,
    download_dir: &str,
) {
    println!();
    println!("{}", style("Configuration:").bold());
    println!("  Creators: {}", creators.join(", "));
    println!("  Mode: {}", download_mode);
    println!("  Directory: {}", download_dir);
    println!();
}
