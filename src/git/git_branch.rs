use crate::error::Error;
use crate::git::git_diff::head;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use git2::{BranchType, Oid};

#[derive(Debug)]
pub struct Branch {
    pub name: String,
    pub last_commit: Oid,
    pub branch_type: BranchType,
}

pub fn checkout_branch(repo_path: &Path, branch_name: &str) -> Result<(), Error> {
    // TODO: Handle remote branch checkout
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

pub fn get_branches(repo_path: &Path) -> Result<Vec<Branch>, Error> {
    let repo = repo(repo_path)?;
    let mut git_branches = repo.branches(Some(git2::BranchType::Local))?.collect::<Vec<_>>();
    let mut remote_branches = repo.branches(Some(git2::BranchType::Remote))?.collect::<Vec<_>>();
    let mut branch_list = Vec::new();
    git_branches.append(&mut remote_branches);

    for git_branch in git_branches {
        let (branch, branch_type) = git_branch?;
        let reference = branch.get();

        let name = reference
            .shorthand()
            .expect("Branch name is not valid UTF-8");
        let commit = reference.peel_to_commit()?;

        branch_list.push(Branch {
            name: name.to_string(),
            last_commit: commit.id(),
            branch_type,
        });
    }
    Ok(branch_list)
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
