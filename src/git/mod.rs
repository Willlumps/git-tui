pub mod commit;
pub mod git_branch;
pub mod git_diff;
pub mod git_log;
pub mod git_status;
pub mod push;
pub mod stage;

use std::path::Path;

use anyhow::Result;
use git2::Repository;

pub fn repo(repo_path: &Path) -> Result<Repository, git2::Error> {
    let repo = Repository::open(repo_path)?;
    Ok(repo)
}
