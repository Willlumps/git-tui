pub mod commit;
pub mod branch;
pub mod diff;
pub mod log;
pub mod status;
pub mod fetch;
pub mod push;
pub mod stage;
pub mod time;

use std::path::Path;

use anyhow::Result;
use git2::Repository;

use crate::error::Error;

use self::commit::create_initial_commit;

pub fn repo(repo_path: &Path) -> Result<Repository, git2::Error> {
    let repo = Repository::open(repo_path)?;
    Ok(repo)
}

pub fn init_empty_repo(repo_path: &Path) -> Result<(), Error> {
    create_initial_commit(repo_path)?;
    Ok(())
}

pub fn is_empty_repo(repo_path: &Path) -> Result<bool, git2::Error> {
    // TODO: Sanity check this since it will be our first attempt at opening
    //       a repository.
    let repo = Repository::open(repo_path)?;

    if repo.is_empty()? {
        Ok(true)
    } else {
        Ok(false)
    }
}
