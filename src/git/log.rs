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
        let mut message_summary = match commit.summary() {
            Some(summary) => summary.to_string(),
            None => String::new(),
        };
        let mut message_body = match commit.body() {
            Some(body) => body.split('\n').map(|line| line.to_string()).collect(),
            None => Vec::new(),
        };
        let time = CommitDate::new(commit.time());

        if message_summary.len() > 70 {
            let index = get_split_index(&message_summary);
            let body = message_summary[index..].trim().to_string();
            message_summary = message_summary[0..index].to_string();
            message_body.insert(0, body);
        }

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

fn get_split_index(message_summary: &str) -> usize {
    // what in tarnation is this?
    let (mut i, mut c) = message_summary
        .char_indices()
        .nth(70)
        .expect("I should just work");
    if c != ' ' {
        loop {
            if i == 55 || c == ' ' {
                break;
            }

            i -= 1;
            c = message_summary.chars().nth(i).expect("I should work, too");
        }
    }

    i
}
