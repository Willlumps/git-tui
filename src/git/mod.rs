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

pub fn repo(repo_path: &Path) -> Result<Repository, git2::Error> {
    let repo = Repository::open(repo_path)?;
    Ok(repo)
}
