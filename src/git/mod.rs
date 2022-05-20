use anyhow::Result;
use git2::Repository;
use std::path::Path;

pub mod commit;
pub mod git_branch;
pub mod git_diff;
pub mod git_log;
pub mod git_status;
pub mod stage;
pub mod push;

pub fn repo(repo_path: &Path) -> Result<Repository> {
    let repo = Repository::init(repo_path)?;
    Ok(repo)
}
