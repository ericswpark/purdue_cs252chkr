use cs252chkr::*;
use std::collections::HashMap;

fn main() {
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
    let commit_counts = get_commit_counts(&repo).expect("Failed to get commit counts");
    let estimates = get_estimate_minutes(&repo).expect("Failed to get estimate minutes");
    let metadata = zip_by_author(commit_counts, estimates);
    for entry in metadata {
        println!(
            "{}: {} commits ({})",
            entry.0,
            entry.1 .0.count,
            get_humanized_minutes(entry.1 .1.time as i64)
        );
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
