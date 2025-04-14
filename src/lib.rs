pub mod constants;
pub mod datetime;
pub mod git;
pub mod structs;

use crate::constants::INSERTION_DELETION_WARNING_RATIO;
use crate::git::{
    get_commit_stats, get_estimate_minutes, get_initial_commit, get_repository, print_partial_logs,
};
use crate::structs::{CommitStats, CommitTime};
use anyhow::Context;

use crate::datetime::{
    get_formatted_time, get_humanized_minutes, get_humanized_time, get_localized_time,
};
use std::collections::HashMap;
use thiserror::Error;

/// Library error type
#[derive(Error, Debug)]
pub enum Error {
    #[error("no commits")]
    NoCommits,
    #[error("can't fetch previous commit time")]
    PreviousCommitTimeError,
    #[error("can't fetch commit time")]
    CommitTimeError,
    #[error(transparent)]
    GitError(#[from] git2::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Shared commit metadata trait for zipping and printing metadata information
pub trait CommitMetadata {
    fn author(&self) -> String;
}

/// Formats and prints commit statistics
///
/// Warning: it is up to the caller to make sure that the stat and time objects belong to the same
/// author!
///
/// * `stat` - commit statistics to print
/// * `time` - time spent information for author
pub fn print_commit_stats(stat: &CommitStats, time: &CommitTime) {
    // Pretty-print author name in border
    for _ in 0..stat.author().len() + 4 {
        print!("=");
    }
    println!();
    println!("| {} |", stat.author());
    for _ in 0..stat.author().len() + 4 {
        print!("=");
    }
    println!();

    // Print basic commit stats
    println!("Commits: {} commits", stat.count);
    println!("Time spent: {}", get_humanized_minutes(time.time as i64));

    // Calculate ratio of insertions and deletions and print LOC stats
    let ratio = stat.deletions as f64 / stat.insertions as f64;
    print!(
        "LOC: +{}/-{} ({:.2}%)",
        stat.insertions,
        stat.deletions,
        ratio * 100.0
    );

    // Warn if LOC ratio is strangely low
    if ratio < INSERTION_DELETION_WARNING_RATIO {
        println!(" (!)");
    } else {
        println!();
    }
}

/// Zip two arrays of CommitMetadata objects into tuples for easy access
///
/// * `vec1` - Vec of objects that implement the CommitMetadata trait
/// * `vec2` - Vec of objects that implement the CommitMetadata trait
pub fn zip_by_author<U, V>(vec1: Vec<U>, vec2: Vec<V>) -> Vec<(String, (U, V))>
where
    U: CommitMetadata,
    V: CommitMetadata,
{
    let mut map: HashMap<String, U> = vec1.into_iter().map(|x| (x.author(), x)).collect();
    let mut result = Vec::new();

    for val2 in vec2 {
        if let Some(val1) = map.remove(&val2.author()) {
            result.push((val1.author(), (val1, val2)));
        }
    }

    result
}

/// Check the given directory's git repository for statistics and information
///
/// * `dir` - directory with git repository to check
/// * `partial_logs` - whether to show git partial logs (`git log -p` equivalent)
/// * `pathspec` - pathspec to consider
pub fn check(dir: &str, partial_logs: bool, pathspec: &str) -> Result<(), Error> {
    let repo = get_repository(dir);
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
    let commit_counts = get_commit_stats(&repo, pathspec).context("Failed to get commit counts")?;
    let estimates = get_estimate_minutes(&repo).context("Failed to get estimate minutes")?;
    let metadata = zip_by_author(commit_counts, estimates);
    for entry in metadata {
        print_commit_stats(&entry.1 .0, &entry.1 .1);
    }

    if partial_logs {
        print_partial_logs(&repo, pathspec).context("Failed to print partial logs")?;
    }

    Ok(())
}
