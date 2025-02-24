use anyhow::{Context, Result};
use clap::Parser;
use cs252chkr::constants::SENTRY_DSN_URL;
use cs252chkr::*;
use sentry::ClientInitGuard;

#[derive(Parser, Debug)]
#[command(name = "cs252chkr")]
#[command(author = "Eric Park (@ericswpark)")]
struct Cli {
    /// Enable bug and crash reports to Sentry
    #[arg(short = 'c', long)]
    enable_crash_reports: bool,
}

fn main() -> Result<()> {
    let _sentry_guard;

    // Get commandline arguments
    let cli = Cli::parse();

    if cli.enable_crash_reports {
        println!("Bug/crash reports to Sentry has been enabled!");
        _sentry_guard = sentry_init();
    }

    // Attempt to open repository in current directory, or start walking up
    let repo = get_repository();
    let initial_commit = get_initial_commit(&repo).context("Failed to get initial commit")?;

    // Fetch initial commit time from repository
    let initial_commit_time_raw = initial_commit.time().seconds();
    let initial_commit_time = get_localized_time(initial_commit_time_raw)
        .context("Failed to get localized time for initial commit")?;
    println!(
        "Initial commit was made at {} ({})",
        get_formatted_time(initial_commit_time),
        get_humanized_time(initial_commit_time)
    );

    // Fetch author metadata (commit count, total session duration) from repository
    let commit_counts = get_commit_stats(&repo).context("Failed to get commit counts")?;
    let estimates = get_estimate_minutes(&repo).context("Failed to get estimate minutes")?;
    let metadata = zip_by_author(commit_counts, estimates);
    for entry in metadata {
        print_commit_stats(&entry.1 .0, &entry.1 .1);
    }

    Ok(())
}

/// Initializes Sentry, a bug report platform
///
/// The returned guard must be placed within a variable that should stay in scope throughout program
/// execution! Otherwise, panics will not be caught by Sentry.
fn sentry_init() -> ClientInitGuard {
    sentry::init((
        SENTRY_DSN_URL,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ))
}
