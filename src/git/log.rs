use crate::error::Error;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use git2::Oid;

#[derive(Clone, Debug, Default)]
pub struct Commit {
    id: String,
    author: String,
    email: String,
    message: String,
}

impl Commit {
    pub fn id(&self) -> &String {
        &self.id
    }

    #[allow(dead_code)]
    pub fn author(&self) -> &String {
        &self.author
    }

    #[allow(dead_code)]
    pub fn email(&self) -> &String {
        &self.email
    }

    pub fn message(&self) -> &String {
        &self.message
    }

    pub fn shorthand_id(&self) -> String {
        self.id[0..8].to_string()
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
            let id = commit.id().to_string();

            let author = match commit.author().name() {
                Some(author) => author.to_string(),
                None => String::new()
            };
            let email = match commit.author().email() {
                Some(email) => email.to_string(),
                None => String::new()
            };
            let message = match commit.summary() {
                Some(summary) => summary.to_string(),
                None => String::new()
            };

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
