use chrono::{DateTime, Local};
use git2::{Commit, Error, Repository};

pub fn get_repository() -> Repository {
    Repository::discover(".").expect("Can't find git repository (attempted traversal to root). Are you sure you're in the project folder?")
}

pub fn get_initial_commit(repo: &Repository) -> Result<Commit, Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::REVERSE)?;

    if let Some(first_commit_id) = revwalk.next() {
        return Ok(repo.find_commit(first_commit_id?)?);
    }
    Err(Error::from_str("This repository does not have any commits!"))
}

pub fn get_localized_time(seconds: i64) -> Option<String> {
    let time_utc = DateTime::from_timestamp(seconds, 0)?;
    let time_local = time_utc.with_timezone(&Local);
    Some(time_local.format("%Y-%m-%d %H:%M:%S").to_string())
}