use crate::error::Error;
use crate::git::diff::head;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use git2::{BranchType, Oid, Repository};

#[derive(Clone, Debug)]
pub struct Branch {
    pub name: String,
    pub last_commit: Oid,
    pub branch_type: BranchType,
    pub time_since_commit: TimeSinceCommit,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSinceCommit(pub u64);

impl From<TimeSinceCommit> for String {
    fn from(time: TimeSinceCommit) -> Self {
        match time.0 {
            x if x < 3600 => format!("{}{}", (x / 60), "m"),
            x if x < 86400 => format!("{}{}", (x / 3600), "hr"),
            x if x < 604_800 => format!("{}{}", (x / 86400), "d"),
            x if x < 2_419_200 => format!("{}{}", (x / 604_800), "wk"),
            x if x < 31_536_000 => format!("{}{}", (x / 2_419_200), "mo"),
            x => format!("{}{}", (x / 31_536_000), "yr"),
        }
    }
}

pub fn checkout_local_branch(repo_path: &Path, branch_name: &str) -> Result<(), Error> {
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

pub fn checkout_remote_branch(repo_path: &Path, remote_branch_name: &str) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let name = remote_branch_name
        .split('/')
        .skip(1)
        .collect::<Vec<&str>>()
        .join("");

    if does_local_branch_exist(&repo, &name) {
        return Err(Error::Git(git2::Error::from_str("Local branch already exists")));
    }

    let (object, _reference) = repo
        .revparse_ext(remote_branch_name)
        .expect("Object not found");
    let commit = object.as_commit().unwrap();

    repo.branch(&name, commit, false)?;

    // Need to change the files in the working directory as well as set the HEAD
    let (object, reference) = repo.revparse_ext(&name).expect("Object not found");
    repo.checkout_tree(&object, None)?;
    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }
    .expect("Failed to set HEAD");

    set_upstream_branch(&repo, &name)?;

    Ok(())
}

pub fn delete_branch(repo_path: &Path, branch_name: &str) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
    branch.delete()?;
    Ok(())
}

pub fn get_branches(repo_path: &Path) -> Result<Vec<Branch>, Error> {
    let repo = repo(repo_path)?;
    let mut branch_list = Vec::new();

    let mut git_branches = repo
        .branches(Some(git2::BranchType::Local))?
        .collect::<Vec<_>>();

    let mut remote_branches = repo
        .branches(Some(git2::BranchType::Remote))?
        .collect::<Vec<_>>();

    git_branches.append(&mut remote_branches);

    for git_branch in git_branches {
        let (branch, branch_type) = git_branch?;
        let reference = branch.get();
        let name = reference
            .shorthand()
            .expect("Branch name is not valid UTF-8");
        let commit = reference.peel_to_commit()?;

        let time_since_commit = time_since_commit(commit.time().seconds() as u64);

        branch_list.push(Branch {
            name: name.to_string(),
            last_commit: commit.id(),
            branch_type,
            time_since_commit,
        });
    }
    Ok(branch_list)
}

fn time_since_commit(seconds: u64) -> TimeSinceCommit {
    let commit_time = std::time::Duration::new(seconds, 0);
    let start = std::time::SystemTime::now();
    let since_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let diff = since_epoch.saturating_sub(commit_time);

    TimeSinceCommit(diff.as_secs())
}

pub fn branch_from_head(repo_path: &Path, new_branch_name: &str) -> Result<(), Error> {
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

fn set_upstream_branch(repo: &Repository, branch_name: &str) -> Result<(), Error> {
    let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
    if branch.upstream().is_err() {
        branch.set_upstream(Some(format!("origin/{}", branch_name).as_str()))?;
    }
    Ok(())
}

fn does_local_branch_exist(repo: &Repository, branch_name: &str) -> bool {
    repo.find_branch(branch_name, BranchType::Local).is_ok()
}
