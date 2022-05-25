use crate::app::ProgramEvent;
use crate::error::Error;

use super::repo;
use anyhow::Result;
use std::{path::Path, sync::mpsc::Sender};

pub struct Branch {
    // This will probably need more fields, though as of now I don't know
    // what those would be.
    pub name: String,
}

impl Branch {
    pub fn checkout_branch(
        &self,
        repo_path: &Path,
        event_sender: Sender<ProgramEvent>,
    ) -> Result<(), git2::Error> {
        let repo = repo(repo_path)?;
        // Need to change the files in the working directory as well as set the HEAD
        let (object, reference) = repo.revparse_ext(&self.name).expect("Object not found");

        if let Err(err) = repo.checkout_tree(&object, None) {
            event_sender
                .send(ProgramEvent::Error(Error::Git(err)))
                .expect("Failed to send");
        } else {
            match reference {
                // gref is an actual reference like branches or tags
                Some(gref) => repo.set_head(gref.name().unwrap()),
                // this is a commit, not a reference
                None => repo.set_head_detached(object.id()),
            }
            .expect("Failed to set HEAD");
        }

        Ok(())
    }
}

pub fn get_branches(repo_path: &Path) -> Result<Vec<Branch>> {
    let repo = repo(repo_path)?;
    let git_branches = repo.branches(Some(git2::BranchType::Local))?;
    let mut branches = Vec::new();

    for git_branch in git_branches {
        let branch = git_branch?.0;
        if let Some(name) = branch.name()? {
            branches.push(Branch {
                name: name.to_string(),
            });
        }
    }
    Ok(branches)
}

pub fn new_branch(
    repo_path: &Path,
    refname: String,
    new_branch_name: String,
) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let (object, _reference) = repo.revparse_ext(&refname).expect("Revspec not found");
    match object.as_commit() {
        Some(commit) => {
            if let Err(err) = repo.branch(&new_branch_name, commit, false) {
                return Err(Error::Git(err));
            }
        }
        None => {
            return Err(Error::Unknown("Object is not a commit".to_string()));
        }
    }
    Ok(())
}
