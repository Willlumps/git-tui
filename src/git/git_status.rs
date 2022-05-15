use anyhow::Result;
use super::repo;

use std::path::Path;

#[derive(Default)]
pub struct GitStatus {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub branch: String,
}

pub fn get_stats(repo_path: &Path) -> Result<GitStatus> {
    let repo = repo(repo_path)?;

    let mut opt = git2::DiffOptions::new();
    let diff = repo.diff_index_to_workdir(None, Some(&mut opt))?;
    let stats = diff.stats()?;
    let status = GitStatus {
        files_changed: stats.files_changed(),
        insertions: stats.insertions(),
        deletions: stats.deletions(),
        branch: head(repo_path)?,
    };

    Ok(status)
}

fn head(repo_path: &Path) -> Result<String> {
    let repo = repo(repo_path)?;
    let head_ref = repo.head()?;
    if let Some(branch_name) = head_ref.shorthand() {
        Ok(String::from(branch_name))
    } else {
        Ok("".to_string())
    }
}


