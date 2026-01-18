//! Statistics reporting.

use console::style;

use crate::download::{DownloadState, GlobalState};

/// Print statistics for a single creator.
pub fn print_creator_stats(state: &DownloadState) {
    let creator_name = state.creator_name.as_deref().unwrap_or("unknown");

    println!();
    println!(
        "{}",
        style(format!("Statistics for {}:", creator_name)).bold()
    );
    println!("  Pictures: {}", state.pic_count);
    println!("  Videos:   {}", state.vid_count);
    println!("  Audio:    {}", state.audio_count);
    println!("  Skipped:  {} (duplicates)", state.duplicate_count());
    println!("  Total:    {} downloaded", state.total_downloaded());
}

/// Print global statistics across all creators.
pub fn print_global_stats(state: &GlobalState) {
    println!();
    println!("{}", style("═".repeat(50)).dim());
    println!("{}", style("Global Statistics:").bold());
    println!("  Creators processed: {}", state.creators_processed);
    if state.creators_failed > 0 {
        println!(
            "  Creators failed:    {}",
            style(state.creators_failed).red()
        );
    }
    println!("  Pictures: {}", state.pic_count);
    println!("  Videos:   {}", state.vid_count);
    println!("  Audio:    {}", state.audio_count);
    println!("  Skipped:  {} (duplicates)", state.duplicate_count);
    println!("  Total:    {} downloaded", state.total_downloaded());
    println!("{}", style("═".repeat(50)).dim());
}

/// Print a summary line for quick viewing.
pub fn print_summary(pics: u64, vids: u64, audio: u64, dupes: u64) {
    println!(
        "Downloaded: {} pics, {} vids, {} audio ({} skipped)",
        style(pics).green(),
        style(vids).green(),
        style(audio).green(),
        style(dupes).yellow()
    );
}
