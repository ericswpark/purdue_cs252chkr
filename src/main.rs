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
    /// Show git partial logs (`git log -p`). Warning: generates a lot of output
    #[arg(short = 'p', long)]
    git_log_partial: bool,
}

fn main() -> Result<()> {
    let _sentry_guard;

    // Get commandline arguments
    let cli = Cli::parse();

    if cli.enable_crash_reports {
        println!("Bug/crash reports to Sentry has been enabled!");
        _sentry_guard = sentry_init();
    }

    // Run checks against repository in current directory
    check(".", cli.git_log_partial)
        .context("Failed to run checks on current directory's git repository")?;

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
