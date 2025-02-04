use chrono::{DateTime, Local, Utc};
use chrono_humanize::HumanTime;
use git2::{Commit, Error, Repository};
use std::collections::HashMap;

pub fn get_repository() -> Repository {
    Repository::discover(".").expect("Can't find git repository (attempted traversal to root). Are you sure you're in the project folder?")
}

pub fn get_initial_commit(repo: &Repository) -> Result<Commit, Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::REVERSE)?;

    if let Some(first_commit_id) = revwalk.next() {
        return repo.find_commit(first_commit_id?);
    }
    Err(Error::from_str(
        "This repository does not have any commits!",
    ))
}

pub fn get_commit_counts(repo: &Repository) -> Result<Vec<(String, usize)>, Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commit_map: HashMap<String, usize> = HashMap::new();

    for oid in revwalk {
        let commit = repo.find_commit(oid?)?;
        let author = commit.author().name().unwrap_or("-").to_string();
        *commit_map.entry(author).or_insert(0) += 1;
    }

    let mut commit_map: Vec<_> = commit_map.into_iter().collect();
    commit_map.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(commit_map)
}

const MAX_COMMIT_DIFF_IN_MINUTES: usize = 120;
const FIRST_COMMIT_ADDITION_IN_MINUTES: usize = 120;

/// Implements git-hours algorithm
pub fn get_estimate_minutes(repo: &Repository) -> Result<Vec<(String, usize)>, Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::REVERSE & git2::Sort::TIME)?;

    let mut commit_map: HashMap<String, (Option<Commit>, usize)> = HashMap::new();

    for oid in revwalk {
        let commit = repo.find_commit(oid?)?;
        let author = commit.author().name().unwrap_or("-").to_string();
        let entry = commit_map.entry(author).or_insert((None, 0));

        // First commit
        if (*entry).0.is_none() {
            *entry = (Some(commit), 120);
            continue;
        }

        // Calculate difference in minutes
        let previous_commit_seconds = entry.0.to_owned().unwrap().time().seconds();
        let previous_commit_time = get_time(previous_commit_seconds).ok_or(Error::from_str("Can't fetch previous commit's time"))?;

        let commit_seconds = commit.time().seconds();
        let commit_time = get_time(commit_seconds).ok_or(Error::from_str("Can't fetch commit's time"))?;

        let time_diff = commit_time - previous_commit_time;
        let time_diff_minutes = time_diff.num_minutes();

        if (time_diff_minutes as usize) < MAX_COMMIT_DIFF_IN_MINUTES {
            entry.1 += time_diff_minutes as usize;
        } else {
            entry.1 += FIRST_COMMIT_ADDITION_IN_MINUTES;
        }
    }

    let mut commit_map: Vec<_> = commit_map.into_iter().map(|e| (e.0, e.1.1)).collect();
    commit_map.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(commit_map)
}

pub fn get_time(seconds: i64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(seconds, 0)
}

pub fn get_localized_time(seconds: i64) -> Option<DateTime<Local>> {
    let time_utc = get_time(seconds)?;
    Some(time_utc.with_timezone(&Local))
}

pub fn get_formatted_time(time: DateTime<Local>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn get_humanized_time(time: DateTime<Local>) -> String {
    HumanTime::from(time - Local::now()).to_string()
}
