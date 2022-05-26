use crate::error::Error;

use super::{git_diff::head, repo};
use anyhow::Result;
use git2::Oid;
use std::path::Path;

#[derive(Debug)]
pub struct Branch {
    pub name: String,
    pub last_commit: Oid,
    // pub last_commit_time: String? Time?
    // pub branch_type??
}

pub fn checkout_branch(
    repo_path: &Path,
    branch_name: &str,
) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    // Need to change the files in the working directory as well as set the HEAD
    let (object, reference) = repo.revparse_ext(branch_name).expect("Object not found");

    repo.checkout_tree(&object, None)?;
    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }
    .expect("Failed to set HEAD");

    Ok(())
}

pub fn get_branches(repo_path: &Path) -> Result<Vec<Branch>> {
    let repo = repo(repo_path)?;
    let git_branches = repo.branches(Some(git2::BranchType::Local))?;
    let mut branches = Vec::new();

    for git_branch in git_branches {
        let branch = git_branch?.0;
        let reference = branch.get();

        let name = reference
            .shorthand()
            .expect("Branch name is not valid UTF-8");
        let commit = reference.peel_to_commit()?;

        branches.push(Branch {
            name: name.to_string(),
            last_commit: commit.id(),
        });
    }
    Ok(branches)
}

pub fn create_branch(repo_path: &Path, new_branch_name: &str) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let head = head(repo_path)?;
    let (object, _reference) = repo.revparse_ext(&head).expect("Revspec not found");
    match object.as_commit() {
        Some(commit) => {
            if let Err(err) = repo.branch(new_branch_name, commit, false) {
                return Err(Error::Git(err));
            }
        }
        None => {
            return Err(Error::Unknown("Object is not a commit".to_string()));
        }
    }
    Ok(())
}
