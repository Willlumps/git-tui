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
    message_summary: String,
    message_body: Vec<String>,
    time: CommitDate,
}

impl Commit {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            author: String::new(),
            email: String::new(),
            message_summary: String::new(),
            message_body: Vec::new(),
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
        let message_summary = match commit.summary() {
            Some(summary) => summary.to_string(),
            None => String::new(),
        };
        let message_body = match commit.body() {
            Some(body) => body.split('\n').map(|line| line.to_string()).collect(),
            None => Vec::new(),
        };
        let time = CommitDate::new(commit.time());

        Self {
            id,
            author,
            email,
            message_summary,
            message_body,
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

    pub fn message_summary(&self) -> &String {
        &self.message_summary
    }

    pub fn message_body(&self) -> &Vec<String> {
        &self.message_body
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
