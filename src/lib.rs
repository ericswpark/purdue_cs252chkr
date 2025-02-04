use chrono::{DateTime, Local};
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

pub fn get_localized_time(seconds: i64) -> Option<DateTime<Local>> {
    let time_utc = DateTime::from_timestamp(seconds, 0)?;
    Some(time_utc.with_timezone(&Local))
}

pub fn get_formatted_time(time: DateTime<Local>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn get_humanized_time(time: DateTime<Local>) -> String {
    HumanTime::from(time - Local::now()).to_string()
}
