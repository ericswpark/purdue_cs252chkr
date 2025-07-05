use anyhow::{Context, Result};
use clap::Parser;
use cs252chkr::constants::LOC_PATHSPEC;
use cs252chkr::*;

#[derive(Parser, Debug)]
#[command(name = "cs252chkr")]
#[command(author = "Eric Park (@ericswpark)")]
struct Cli {
    /// Show git partial logs (`git log -p`). Warning: generates a lot of output
    #[arg(short = 'p', long)]
    git_log_partial: bool,
    /// Ignore pathspec when considering source files
    #[arg(long)]
    ignore_pathspec: bool,
    #[arg(conflicts_with = "ignore_pathspec")]
    pathspec: Vec<String>,
}

fn main() -> Result<()> {
    // Get commandline arguments
    let cli = Cli::parse();

    let pathspec: String = if cli.ignore_pathspec {
        "".to_string()
    } else if !cli.pathspec.is_empty() {
        cli.pathspec.join(" ")
    } else {
        LOC_PATHSPEC.to_string()
    };

    // Run checks against repository in current directory
    check(".", cli.git_log_partial, &pathspec)
        .context("Failed to run checks on current directory's git repository")?;

    Ok(())
}
