use cs252chkr::*;

fn main() {
    // Attempt to open repository in current directory, or start walking up
    let repo = get_repository();
    let initial_commit = get_initial_commit(&repo).expect("Failed to get initial commit");
    let commit_counts = get_commit_counts(&repo).expect("Failed to get commit counts");
    let estimates = get_estimate_minutes(&repo).expect("Failed to get estimate minutes");

    // Fetch metadata from repository
    let initial_commit_time_raw = initial_commit.time().seconds();
    let initial_commit_time = get_localized_time(initial_commit_time_raw).unwrap();

    println!(
        "Initial commit was made at {} ({})",
        get_formatted_time(initial_commit_time),
        get_humanized_time(initial_commit_time)
    );

    for entry in commit_counts {
        println!("{}: {} commits", entry.0, entry.1);
    }

    for entry in estimates {
        println!("{}: {} minutes", entry.0, entry.1);
    }
}
