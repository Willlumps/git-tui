use super::repo;
use anyhow::Result;
use git2::StatusOptions;

use std::path::Path;

#[derive(Clone, Debug)]
pub enum StatusType {
    Added,
    Deleted,
    Modified,
    Renamed,
    Typechanged,
    Conflicted,
}

#[derive(Clone, Debug)]
pub struct FileStatus {
    pub path: String,
    pub status_type: StatusType,
}

pub fn get_modified_files(repo_path: &Path) -> Result<Vec<FileStatus>> {
    let repo = repo(repo_path)?;
    let mut files: Vec<FileStatus> = Vec::new();

    let statuses = repo.statuses(Some(&mut StatusOptions::new()))?;
    for status in statuses.iter() {
        if let Some(file_path) = status.path() {
            if let Some(path) = Path::new(file_path).file_name() {
                let file_name = path.to_string_lossy().to_string();

                files.push(FileStatus {
                    path: file_name,
                    status_type: StatusType::Modified,
                });
            }
        }
    }
    Ok(files)
}
