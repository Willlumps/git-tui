use super::repo;
use anyhow::Result;
use git2::Status;
use git2::StatusOptions;

use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
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

pub fn get_file_status(repo_path: &Path) -> Result<Vec<FileStatus>> {
    let repo = repo(repo_path)?;
    let mut files: Vec<FileStatus> = Vec::new();

    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .renames_head_to_index(true)
        .update_index(true);
    let statuses = repo.statuses(Some(&mut options))?;

    for status in statuses.iter() {
        let s = status.status();
        if let Some(file_path) = status.path() {
            if let Some(path) = Path::new(file_path).file_name() {
                let file_name = path.to_string_lossy().to_string();

                files.push(FileStatus {
                    path: file_name,
                    status_type: StatusType::from(s),
                });
            }
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
        }else {
            StatusType::Modified
        }
    }
}

impl From<StatusType> for char {
    fn from(status_type: StatusType) -> char {
        use StatusType::*;
        match status_type {
            Added => 'A',
            Deleted => 'D',
            Modified => 'M',
            Renamed => 'R',
            Typechanged => 'T',
            Conflicted => 'C',
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_deleted() {
        let mut status: Status = Status::empty();

        status.insert(Status::WT_DELETED);
        assert!(status.is_wt_deleted());
        assert_eq!(StatusType::from(status), StatusType::Deleted);

        status.insert(Status::INDEX_DELETED);
        assert!(status.is_index_deleted());
        assert_eq!(StatusType::from(status), StatusType::Deleted);
    }

    #[test]
    fn from_new() {
        let mut status: Status = Status::empty();

        status.insert(Status::WT_NEW);
        assert!(status.is_wt_new());
        assert_eq!(StatusType::from(status), StatusType::Added);

        status.insert(Status::INDEX_NEW);
        assert!(status.is_index_new());
        assert_eq!(StatusType::from(status), StatusType::Added);
    }

    #[test]
    fn from_modified() {
        let mut status: Status = Status::empty();

        status.insert(Status::WT_MODIFIED);
        assert!(status.is_wt_modified());
        assert_eq!(StatusType::from(status), StatusType::Modified);

        status.insert(Status::INDEX_MODIFIED);
        assert!(status.is_index_modified());
        assert_eq!(StatusType::from(status), StatusType::Modified);
    }
}
