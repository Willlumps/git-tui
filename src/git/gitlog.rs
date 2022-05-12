use git2::Oid;
use git2::Repository;

use std::error::Error;

#[derive(Clone, Debug)]
pub struct Commit {
    pub id: String,
    pub author: String,
    pub email: String,
    pub message: String,
}

impl Commit {
    pub fn get_id(&self) -> &String {
        &self.id
    }

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

pub struct GitLog<'src> {
    pub repo_path: &'src str,
    pub history: Vec<Commit>,
}

impl<'src> GitLog<'src> {
    pub fn new(repo_path: &'src str) -> Self {
        Self {
            repo_path,
            history: Vec::new(),
        }
    }

    pub fn get_history(&mut self) {
        self.fetch_history().unwrap();
    }

    fn fetch_history(&mut self) -> Result<(), Box<dyn Error>> {
        self.history.clear();
        let repo = match Repository::init(self.repo_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to init: {}", e),
        };

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

                self.history.push(Commit {
                    id,
                    author,
                    email,
                    message,
                });
            }
        }

        Ok(())
    }
}
