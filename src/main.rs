use cs252chkr::*;
use std::collections::HashMap;
use std::hash::Hash;

fn main() {
    // Attempt to open repository in current directory, or start walking up
    let repo = get_repository();
    let initial_commit = get_initial_commit(&repo).expect("Failed to get initial commit");
    let commit_counts = get_commit_counts(&repo).expect("Failed to get commit counts");
    let estimates = get_estimate_minutes(&repo).expect("Failed to get estimate minutes");
    let metadata = zip_by_author(commit_counts, estimates);

    // Fetch metadata from repository
    let initial_commit_time_raw = initial_commit.time().seconds();
    let initial_commit_time = get_localized_time(initial_commit_time_raw).unwrap();

    println!(
        "Initial commit was made at {} ({})",
        get_formatted_time(initial_commit_time),
        get_humanized_time(initial_commit_time)
    );

    for entry in metadata {
        println!(
            "{}: {} commits ({})",
            entry.0,
            entry.1 .0.count,
            get_humanized_minutes(entry.1 .1.time as i64)
        );
    }
}

fn zip_by_author<T, U, V>(vec1: Vec<U>, vec2: Vec<V>) -> Vec<(T, (U, V))>
where
    T: Eq + Hash,
    U: CommitMetadata,
    V: CommitMetadata,
{
    let mut map: HashMap<T, U> = vec1.into_iter().map(|x| (x.author(), x)).collect();
    let mut result = Vec::new();

    for (key, val2) in vec2 {
        if let Some(val1) = map.remove(&key) {
            result.push((key, (val1, val2)));
        }
    }

    result
}
