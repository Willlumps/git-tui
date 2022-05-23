use crate::app::{ProgramEvent, ErrorType};

use super::repo;
use anyhow::Result;
use std::{path::Path, sync::mpsc::Sender};

pub struct Branch {
    // This will probably need more fields, though as of now I don't know
    // what those would be.
    pub name: String,
}

impl Branch {
    pub fn checkout_branch(&self, repo_path: &Path, event_sender: Sender<ProgramEvent>) -> Result<(), git2::Error> {
        let repo = repo(repo_path)?;
        // Need to change the files in the working directory as well as set the HEAD
        let (object, reference) = repo.revparse_ext(&self.name).expect("Object not found");
        if let Err(err) = repo.checkout_tree(&object, None) {
            event_sender.send(ProgramEvent::Error(ErrorType::GitError(err))).expect("Failed to send");
            return Ok(());
        }

        match reference {
            // gref is an actual reference like branches or tags
            Some(gref) => repo.set_head(gref.name().unwrap()),
            // this is a commit, not a reference
            None => repo.set_head_detached(object.id()),
        }
        .expect("Failed to set HEAD");
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

