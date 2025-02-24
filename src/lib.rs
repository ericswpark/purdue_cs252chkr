pub mod constants;
pub mod git;
pub mod structs;

use crate::constants::INSERTION_DELETION_WARNING_RATIO;
use crate::git::{get_commit_stats, get_estimate_minutes, get_initial_commit, get_repository};
use crate::structs::{CommitStats, CommitTime};
use anyhow::Context;
use chrono::{DateTime, Duration, Local, Utc};
use chrono_humanize::Accuracy::Precise;
use chrono_humanize::HumanTime;
use chrono_humanize::Tense::Present;
use std::collections::HashMap;
use thiserror::Error;

/// Library error type
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GitError(#[from] git2::Error),
    #[error("unknown error")]
    Unknown,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Shared commit metadata trait for zipping and printing metadata information
pub trait CommitMetadata {
    fn author(&self) -> String;
}

/// Get DateTime object from seconds
///
/// * `seconds` - seconds to convert into DateTime
pub fn get_time(seconds: i64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(seconds, 0)
}

/// Get localized time from seconds
///
/// * `seconds` - seconds to convert into localized DateTime
pub fn get_localized_time(seconds: i64) -> Option<DateTime<Local>> {
    let time_utc = get_time(seconds)?;
    Some(time_utc.with_timezone(&Local))
}

/// Get time formatted into a human-readable format from a localized DateTime
///
/// * `time` - localized DateTime to get formatted human-readable date/time from
pub fn get_formatted_time(time: DateTime<Local>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Get "humanized" (X minutes ago, X hours ago) time from a localized DateTime
///
/// * `time` - localized DateTime to get "humanized" date/time from
pub fn get_humanized_time(time: DateTime<Local>) -> String {
    HumanTime::from(time - Local::now()).to_string()
}

/// Get "humanized" (X hours X minutes) time from minutes
///
/// * `minutes` - minutes to convert into "humanized" time
pub fn get_humanized_minutes(minutes: i64) -> String {
    let duration = Duration::minutes(minutes);
    get_humanized_duration(&duration)
}

/// Get "humanized" (X hours X minutes) time from duration
///
/// * `duration` - Duration reference to convert into "humanized" time
pub fn get_humanized_duration(duration: &Duration) -> String {
    HumanTime::from(*duration).to_text_en(Precise, Present)
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
pub fn check(dir: &str) -> Result<(), Error> {
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
    let commit_counts = get_commit_stats(&repo).context("Failed to get commit counts")?;
    let estimates = get_estimate_minutes(&repo).context("Failed to get estimate minutes")?;
    let metadata = zip_by_author(commit_counts, estimates);
    for entry in metadata {
        print_commit_stats(&entry.1 .0, &entry.1 .1);
    }

    Ok(())
}
