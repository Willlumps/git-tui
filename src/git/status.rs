use std::path::Path;

use anyhow::Result;
use git2::Status;
use git2::StatusOptions;

use crate::git::repo;

#[derive(Clone, Debug, PartialEq)]
pub enum StatusType {
    Added,
    Deleted,
    IndexModified,
    WtModified,
    Renamed,
    Typechanged,
    Conflicted,
    Unmodified,
}

#[derive(Clone, Debug)]
pub struct FileStatus {
    pub path: String,
    pub status_type: StatusType,
    pub status_loc: StatusLoc,
}

#[derive(Clone, Debug)]
pub enum StatusLoc {
    None,
    Index,
    WorkingDirectory,
    WorkingDirectoryAndIndex,
}

pub fn get_file_status(repo_path: &Path) -> Result<Vec<FileStatus>> {
    let repo = repo(repo_path)?;
    let mut files: Vec<FileStatus> = Vec::new();

    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .renames_head_to_index(true)
        .update_index(true)
        .recurse_untracked_dirs(true);
    let statuses = repo.statuses(Some(&mut options))?;

    for status in statuses.iter() {
        let s = status.status();
        if let Some(file_path) = status.path() {
            let path = Path::new(file_path).to_string_lossy().to_string();

            files.push(FileStatus {
                path,
                status_type: StatusType::from(s),
                status_loc: StatusLoc::from(s),
            });
        }
    }
    Ok(files)
}

impl From<Status> for StatusType {
    fn from(status: Status) -> StatusType {
        if status.is_wt_new() || status.is_index_new() {
            StatusType::Added
        } else if status.is_wt_deleted() || status.is_index_deleted() {
            StatusType::Deleted
        } else if status.is_wt_renamed() || status.is_index_renamed() {
            StatusType::Renamed
        } else if status.is_wt_typechange() || status.is_index_typechange() {
            StatusType::Typechanged
        } else if status.is_conflicted() {
            StatusType::Conflicted
        } else if status.is_index_modified() {
            StatusType::IndexModified
        } else {
            StatusType::WtModified
        }
    }
}

impl From<Status> for StatusLoc {
    fn from(status: Status) -> StatusLoc {
        let bits = status.bits();
        if bits < 128 {
            StatusLoc::Index
        } else if bits == 258 {
            StatusLoc::WorkingDirectoryAndIndex
        } else {
            StatusLoc::WorkingDirectory
        }
    }
}

impl From<StatusType> for char {
    fn from(status_type: StatusType) -> char {
        use StatusType::*;
        match status_type {
            Added => 'A',
            Deleted => 'D',
            IndexModified | WtModified => 'M',
            Renamed => 'R',
            Typechanged => 'T',
            Conflicted => 'C',
            Unmodified => ' ',
        }
    }
}
