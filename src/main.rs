use cs252chkr::constants::{INSERTION_DELETION_WARNING_RATIO, SENTRY_DSN_URL};
use cs252chkr::*;
use sentry::ClientInitGuard;
use std::collections::HashMap;

fn main() {
    // Initialize Sentry - bug and crash report
    let _guard = sentry_init();

    // Attempt to open repository in current directory, or start walking up
    let repo = get_repository();
    let initial_commit = get_initial_commit(&repo).expect("Failed to get initial commit");

    // Fetch initial commit time from repository
    let initial_commit_time_raw = initial_commit.time().seconds();
    let initial_commit_time = get_localized_time(initial_commit_time_raw).unwrap();
    println!(
        "Initial commit was made at {} ({})",
        get_formatted_time(initial_commit_time),
        get_humanized_time(initial_commit_time)
    );

    // Fetch author metadata (commit count, total session duration) from repository
    let commit_counts = get_commit_stats(&repo).expect("Failed to get commit counts");
    let estimates = get_estimate_minutes(&repo).expect("Failed to get estimate minutes");
    let metadata = zip_by_author(commit_counts, estimates);
    for entry in metadata {
        print_commit_stats(&entry.1 .0, &entry.1 .1);
    }
}

/// Formats and prints commit statistics
///
/// Warning: it is up to the caller to make sure that the stat and time objects belong to the same
/// author!
///
/// * `stat` - commit statistics to print
/// * `time` - time spent information for author
fn print_commit_stats(stat: &CommitStats, time: &CommitTime) {
    for _ in 0..stat.author().len() + 4 {
        print!("=");
    }
    println!();
    println!("| {} |", stat.author());
    for _ in 0..stat.author().len() + 4 {
        print!("=");
    }
    println!();

    println!("Commits: {} commits", stat.count);
    println!("Time spent: {}", get_humanized_minutes(time.time as i64));

    let ratio = stat.deletions as f64 / stat.insertions as f64;
    print!(
        "LOC: +{}/-{} ({:.2}%)",
        stat.insertions,
        stat.deletions,
        ratio * 100.0
    );

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
fn zip_by_author<U, V>(vec1: Vec<U>, vec2: Vec<V>) -> Vec<(String, (U, V))>
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
