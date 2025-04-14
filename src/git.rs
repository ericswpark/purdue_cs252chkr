use crate::constants::{
    CS252_USER_NAME, MAX_SESSION_IDLE_IN_MINUTES, SESSION_START_ADDITION_IN_MINUTES,
};
use crate::datetime::{get_formatted_time, get_localized_time, get_time};
use crate::structs::{CommitStats, CommitTime};
use crate::Error;
use crate::Error::{CommitTimeError, NoCommits, PreviousCommitTimeError};
use color_print::{cprint, cprintln};
use git2::{Commit, DiffFormat, DiffOptions, Repository};
use std::collections::HashMap;

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

    Err(NoCommits)
}

/// Get the commit statistics for each author in the given repository
///
/// The CS252 user will be filtered out!
///
/// * `repo` - Repository reference to find commits and authors from
/// * `pathspec` - pathspec to consider
pub fn get_commit_stats(repo: &Repository, pathspec: &str) -> Result<Vec<CommitStats>, Error> {
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
        let diff_stats = get_diff_stats(repo, &commit, pathspec).unwrap_or((0, 0));

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
/// * `pathspec` - pathspec to consider
fn get_diff_stats(repo: &Repository, commit: &Commit, pathspec: &str) -> Option<(usize, usize)> {
    let parent_commit = commit.parent(0).ok()?;

    let commit_tree = commit.tree().ok()?;
    let parent_tree = parent_commit.tree().ok()?;
    let mut diff_options = DiffOptions::new();
    add_pathspecs(&mut diff_options, pathspec);

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
        let previous_commit_time =
            get_time(previous_commit_seconds).ok_or(PreviousCommitTimeError)?;

        // Get time of the current commit
        let commit_seconds = commit.time().seconds();
        let commit_time = get_time(commit_seconds).ok_or(CommitTimeError)?;

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

/// Prints git partial logs for each commit in the supplied repository
///
/// This function is equivalent to git's `log -p` subcommand.
///
/// * `repo` - repository to print the commit partial logs from
/// * `pathspec` - pathspec to consider
pub fn print_partial_logs(repo: &Repository, pathspec: &str) -> Result<(), Error> {
    // Create git revwalk to iterate over the commits in the provided repository
    let mut revwalk = repo.revwalk()?;
    // Set options for iteration
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::REVERSE & git2::Sort::TIME)?;

    // For each commit ID found during iteration...
    for oid in revwalk {
        let commit = repo.find_commit(oid?)?;

        // Print additional metadata about the commit like git does
        cprintln!("<yellow>commit {}</yellow>", commit.id());
        let date = get_localized_time(commit.time().seconds()).unwrap();
        println!("Date: {}", get_formatted_time(date));

        let stats = get_diff_stats(repo, &commit, pathspec);
        if let Some(stats) = stats {
            cprintln!("<green>Insertions: {}</green>", stats.0);
            cprintln!("<red>Deletions: {}</red>", stats.1);
        } else {
            eprintln!("Warning: failed to get statistics for this commit.");
        }

        let parent_commit = commit.parent(0);
        if parent_commit.is_err() {
            // Initial commit with no parent
            continue;
        }
        let parent_commit = parent_commit?;

        let commit_tree = commit.tree()?;
        let parent_tree = parent_commit.tree()?;
        let mut diff_options = DiffOptions::new();
        add_pathspecs(&mut diff_options, pathspec);

        let diff = repo.diff_tree_to_tree(
            Some(&parent_tree),
            Some(&commit_tree),
            Some(&mut diff_options),
        )?;
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                ' ' => {
                    print!("{}", line.origin());
                    print!("{}", std::str::from_utf8(line.content()).unwrap());
                }
                '+' => {
                    cprint!("<green>{}</green>", line.origin());
                    cprint!(
                        "<green>{}</green>",
                        std::str::from_utf8(line.content()).unwrap()
                    );
                }
                '-' => {
                    cprint!("<red>{}</red>", line.origin());
                    cprint!(
                        "<red>{}</red>",
                        std::str::from_utf8(line.content()).unwrap()
                    );
                }
                _ => {}
            }
            true
        })?;

        // Print empty line to distinguish with next commit
        println!();
    }
    Ok(())
}

fn add_pathspecs(opt: &mut DiffOptions, pathspec: &str) {
    for pattern in pathspec.split(' ') {
        opt.pathspec(pattern);
    }
}
