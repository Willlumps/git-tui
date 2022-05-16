use anyhow::Result;
use git2::StatusOptions;
use super::repo;

use std::path::Path;

pub enum StatusType {
    Added,
    Deleted,
    Modified,
    Renamed,
    Typechanged,
    Conflicted,
}

pub struct FileStatus {
    pub path: String,
    pub status_type: StatusType,
}

pub fn get_modified_files(repo_path: &Path) -> Result<Vec<String>> {
    let repo = repo(repo_path)?;
    let mut files: Vec<String> = Vec::new();

    let statuses = repo.statuses(Some(&mut StatusOptions::new()))?;
    for status in statuses.iter() {
        if let Some(path) = status.path() {
            let test = path.split('/');
            let test2 = test.last().unwrap_or("");
            files.push(test2.to_string());
        }
    }
    Ok(files)
}

