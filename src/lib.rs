pub mod constants;
pub mod structs;

use constants::{MAX_SESSION_IDLE_IN_MINUTES, SESSION_START_ADDITION_IN_MINUTES};

use crate::constants::{CS252_USER_NAME, INSERTION_DELETION_WARNING_RATIO, LOC_PATHSPEC};
use crate::structs::{CommitStats, CommitTime};
use anyhow::{anyhow, Context};
use chrono::{DateTime, Duration, Local, Utc};
use chrono_humanize::Accuracy::Precise;
use chrono_humanize::HumanTime;
use chrono_humanize::Tense::Present;
use git2::{Commit, DiffOptions, Repository};
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

/// Returns a git2 Repository object pointing to the repository in the given directory (or the
/// parent directories of the given directory)
pub fn get_repository(dir: &str) -> Repository {
    Repository::discover(dir).expect("Given folder (or parent folders) is not a git repository!")
}

/// Gets the initial commit from the provided repository, ignoring the CS252 user
///
/// * `repo` - Repository reference to fetch the initial commit from
pub fn get_initial_commit(repo: &Repository) -> Result<Commit, Error> {
    // Create git revwalk to iterate over the commits in the provided repository
    let mut revwalk = repo.revwalk()?;
    // Set options for iteration
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::REVERSE)?;

    // Checks if the first commit exists or return error if there are no commits
    for first_commit_id in revwalk {
        // Find commit using commit ID
        let commit = repo.find_commit(first_commit_id?)?;

        // If the author is the CS252 initial commit, skip it
        if commit.author().name().unwrap() == CS252_USER_NAME {
            continue;
        }

        return Ok(commit);
    }

    Err(Error::from(anyhow!(
        "This repository does not have any commits!"
    )))
}

/// Get the commit statistics for each author in the given repository
///
/// The CS252 user will be filtered out!
///
/// * `repo`: Repository reference to find commits and authors from
pub fn get_commit_stats(repo: &Repository) -> Result<Vec<CommitStats>, Error> {
    // Create git revwalk to iterate over the commits in the provided repository
    let mut revwalk = repo.revwalk()?;
    // Set options for iteration
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commit_map: HashMap<String, CommitStats> = HashMap::new();

    // For each commit ID found during iteration...
    for oid in revwalk {
        // Find the commit using the ID
        let commit = repo.find_commit(oid?)?;
        // Get the author name (or set as '-' if there is no authorship info associated with the
        // commit
        let author = commit.author().name().unwrap_or("-").to_string();

        // Get stats
        let diff_stats = get_diff_stats(repo, &commit).unwrap_or((0, 0));

        // Insert count into commit_map
        let entry = commit_map.entry(author.clone()).or_insert(CommitStats {
            author,
            count: 0,
            insertions: 0,
            deletions: 0,
        });

        entry.count += 1;
        entry.insertions += diff_stats.0;
        entry.deletions += diff_stats.1;
    }

    // Collect entries from HashMap into array
    let mut commit_map: Vec<_> = commit_map.into_iter().map(|x| x.1).collect();

    // Filter out the CS 252 user
    commit_map.retain(|x| x.author != CS252_USER_NAME);

    // Sort by number of commits (descending order)
    commit_map.sort_by(|a, b| b.count.cmp(&a.count));

    Ok(commit_map)
}

/// Gets the commit diff statistics (insertions and deletions)
///
/// If there is no parent commit, or if the associated tree cannot be fetched, None will be
/// returned.
///
/// * `repo` - Repository reference where `commit` is
/// * `commit` - Commit reference of the commit that resides in `repo` where the diff stats should
///              be extracted from
fn get_diff_stats(repo: &Repository, commit: &Commit) -> Option<(usize, usize)> {
    let parent_commit = commit.parent(0).ok()?;

    let commit_tree = commit.tree().ok()?;
    let parent_tree = parent_commit.tree().ok()?;
    let mut diff_options = DiffOptions::new();
    diff_options.pathspec(LOC_PATHSPEC);

    let diff = repo
        .diff_tree_to_tree(
            Some(&parent_tree),
            Some(&commit_tree),
            Some(&mut diff_options),
        )
        .ok()?;
    let stats = diff.stats().ok()?;

    Some((stats.insertions(), stats.deletions()))
}

/// Get the estimate of working hours spent on the repository
///
/// This function uses and implements the git-hours algorithm found here:
/// https://github.com/kimmobrunfeldt/git-hours
///
/// * `repo` - Repository reference to generate estimate from
pub fn get_estimate_minutes(repo: &Repository) -> Result<Vec<CommitTime>, Error> {
    // Create git revwalk to iterate over the commits in the provided repository
    let mut revwalk = repo.revwalk()?;
    // Set options for iteration
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::REVERSE & git2::Sort::TIME)?;

    let mut commit_map: HashMap<String, (Option<Commit>, usize)> = HashMap::new();

    // For each commit ID found during iteration...
    for oid in revwalk {
        // Find the commit using the ID
        let commit = repo.find_commit(oid?)?;
        // Get the author name (or set as '-' if there is no authorship info associated with the
        // commit
        let author = commit.author().name().unwrap_or("-").to_string();
        // Get or set entry to compare author information
        let entry = commit_map.entry(author).or_insert((None, 0));

        // First commit made by author
        if entry.0.is_none() {
            *entry = (Some(commit), SESSION_START_ADDITION_IN_MINUTES);
            continue;
        }

        // Get time of the previous commit made by this author
        let previous_commit_seconds = entry.0.to_owned().unwrap().time().seconds();
        let previous_commit_time = get_time(previous_commit_seconds)
            .ok_or(Error::from(anyhow!("Can't fetch previous commit time")))?;

        // Get time of the current commit
        let commit_seconds = commit.time().seconds();
        let commit_time =
            get_time(commit_seconds).ok_or(Error::from(anyhow!("Can't fetch commit time")))?;

        // Get time difference between the two commits above
        let time_diff = previous_commit_time - commit_time;
        let time_diff_minutes = time_diff.num_minutes().abs();

        // If the difference counts as one session...
        if (time_diff_minutes as usize) < MAX_SESSION_IDLE_IN_MINUTES {
            // ...add the duration as the time worked on during this session
            entry.1 += time_diff_minutes as usize;
        } else {
            // Start of a new session, add session start minutes
            entry.1 += SESSION_START_ADDITION_IN_MINUTES;
        }

        // Save current commit reference to compare in next iteration
        entry.0 = Some(commit);
    }

    // Collect all entries in HashMap into array
    let mut commit_map: Vec<_> = commit_map
        .into_iter()
        .map(|e| CommitTime {
            author: e.0,
            time: e.1 .1,
        })
        .collect();
    // Sort by number of minutes worked (in descending order)
    commit_map.sort_by(|a, b| b.time.cmp(&a.time));

    Ok(commit_map)
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
