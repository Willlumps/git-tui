use crate::error::Error;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use git2::Oid;

#[derive(Clone, Debug)]
pub struct Commit {
    id: String,
    author: String,
    email: String,
    message: String,
}

impl Commit {
    pub fn get_id(&self) -> &String {
        &self.id
    }

    #[allow(dead_code)]
    pub fn get_author(&self) -> &String {
        &self.author
    }

    #[allow(dead_code)]
    pub fn get_email(&self) -> &String {
        &self.email
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}

pub fn fetch_history(repo_path: &Path) -> Result<Vec<Commit>, Error> {
    let repo = repo(repo_path)?;

    let mut history: Vec<Commit> = Vec::new();
    let mut revwalk = repo.revwalk()?;
    revwalk.reset()?;
    revwalk.push_head()?;

    let oids: Vec<Result<Oid, git2::Error>> = revwalk.collect();
    for oid in oids {
        if oid.is_ok() {
            let commit = repo.find_commit(oid.unwrap())?;
            // TODO: Better error handling to avoid unwrapping these options
            //       and fix whatever the hell is going on down here.
            let id = commit.id().to_string()[0..8].to_string();
            let author = commit.author().name().unwrap().to_string();
            let email = commit.author().email().unwrap().to_string();
            let message = commit.summary().unwrap().to_string();

            history.push(Commit {
                id,
                author,
                email,
                message,
            });
        }
    }

    Ok(history)
}
