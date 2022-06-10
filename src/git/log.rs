use crate::error::Error;
use crate::git::repo;
use crate::git::time::CommitDate;

use std::path::Path;

use anyhow::Result;
use git2::Oid;

#[derive(Clone, Debug)]
pub struct Commit {
    id: String,
    author: String,
    email: String,
    message: String,
    time: CommitDate,
}

impl Commit {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            author: String::new(),
            email: String::new(),
            message: String::new(),
            time: CommitDate::new(git2::Time::new(0, 0)),
        }
    }

    pub fn from_git_commit(commit: git2::Commit) -> Self {
        let id = commit.id().to_string();

        let author = match commit.author().name() {
            Some(author) => author.to_string(),
            None => String::new(),
        };
        let email = match commit.author().email() {
            Some(email) => email.to_string(),
            None => String::new(),
        };
        let message = match commit.summary() {
            Some(summary) => summary.to_string(),
            None => String::new(),
        };
        let time = CommitDate::new(commit.time());

        Self {
            id,
            author,
            email,
            message,
            time,
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn author(&self) -> &String {
        &self.author
    }

    pub fn email(&self) -> &String {
        &self.email
    }

    pub fn message(&self) -> &String {
        &self.message
    }

    pub fn shorthand_id(&self) -> String {
        self.id[0..8].to_string()
    }

    pub fn time(&self) -> &CommitDate {
        &self.time
    }
}

pub fn fetch_history(repo_path: &Path) -> Result<Vec<Commit>, Error> {
    let repo = repo(repo_path)?;

    if repo.is_empty()? {
        return Ok(Vec::new());
    }

    let mut history: Vec<Commit> = Vec::new();
    let mut revwalk = repo.revwalk()?;

    revwalk.reset()?;
    revwalk.push_head()?;

    let oids: Vec<Result<Oid, git2::Error>> = revwalk.collect();
    for oid in oids {
        if oid.is_ok() {
            let commit = repo.find_commit(oid.unwrap())?;
            history.push(Commit::from_git_commit(commit));
        }
    }

    Ok(history)
}
