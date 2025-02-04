use cs252chkr::*;

fn main() {
    // Attempt to open repository in current directory, or start walking up
    let repo = get_repository();
    let initial_commit = get_initial_commit(&repo).expect("Failed to get initial commit");

    // Fetch
    let initial_commit_time_raw = initial_commit.time().seconds();
    let initial_commit_time_str = get_localized_time(initial_commit_time_raw).expect("Invalid initial commit time");

    println!("Initial commit was made at {}", initial_commit_time_str);
}
