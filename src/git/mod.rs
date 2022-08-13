pub mod branch;
pub mod commit;
pub mod diff;
pub mod fetch;
pub mod log;
pub mod push;
pub mod stage;
pub mod status;
pub mod time;

use std::path::Path;

use anyhow::Result;
use git2::{Repository, RepositoryOpenFlags};

use crate::error::Error;

use self::commit::create_initial_commit;

pub fn repo(repo_path: &Path) -> Result<Repository, git2::Error> {
    let repo = Repository::open_ext(repo_path, RepositoryOpenFlags::empty(), Vec::<&Path>::new())?;
    Ok(repo)
}

pub fn init_new_repo(repo_path: &Path) -> Result<(), Error> {
    Repository::init(&repo_path)?;
    create_initial_commit(repo_path)?;
    Ok(())
}

pub fn is_empty_repo(repo_path: &Path) -> Result<bool, git2::Error> {
    let repo = Repository::open_ext(repo_path, RepositoryOpenFlags::empty(), Vec::<&Path>::new())?;

    if repo.is_empty()? {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn is_repo(repo_path: &Path) -> bool {
    Repository::open_ext(repo_path, RepositoryOpenFlags::empty(), Vec::<&Path>::new()).is_ok()
}
