use crate::error::Error;

use super::repo;
use anyhow::Result;
use std::path::Path;

pub fn stage_file(repo_path: &Path, file_path: &str) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let path = Path::new(file_path);
    let mut index = repo.index()?;

    index.add_path(path)?;
    index.write()?;

    Ok(())
}

pub fn unstage_file(repo_path: &Path, file_path: &str) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let path = Path::new(file_path);

    if let Some(head) = repo.head()?.target() {
        let obj = repo.find_object(head, Some(git2::ObjectType::Commit))?;
        repo.reset_default(Some(&obj), &[path])?;
    }
    Ok(())
}

pub fn stage_all(repo_path: &Path) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

pub fn unstage_all(repo_path: &Path) -> Result<(), Error> {
    let repo = repo(repo_path)?;

    if let Some(head) = repo.head()?.target() {
        let obj = repo.find_object(head, Some(git2::ObjectType::Commit))?;
        repo.reset_default(Some(&obj), ["*"].iter())?;
    }
    Ok(())
}
