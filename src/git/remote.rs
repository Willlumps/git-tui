use std::path::Path;
use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;
use git2::string_array::StringArray;
use git2::PushOptions;

use crate::error::Error;
use crate::git::branch::set_upstream_branch;
use crate::git::callbacks::create_remote_callbacks;
use crate::git::diff::head;
use crate::git::repo;

pub fn add_remote(repo_path: &Path, name: &str, url: &str) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    repo.remote(name, url)?;

    Ok(())
}

pub fn get_remotes(repo_path: &Path) -> Result<StringArray, Error> {
    let repo = repo(repo_path)?;
    let remotes = repo.remotes()?;
    Ok(remotes)
}

pub fn push(
    repo_path: &Path,
    progress_sender: Sender<usize>,
    remote: &str,
    retry_count: Arc<Mutex<usize>>,
) -> Result<(), Error> {
    let repo = repo(repo_path)?;

    let mut remote = repo.find_remote(remote)?;
    let head = head(repo_path)?;
    let refspec = format!("refs/heads/{}", head);

    let mut options = PushOptions::new();
    let callbacks = create_remote_callbacks(progress_sender, Some(retry_count));
    options.remote_callbacks(callbacks);

    remote.push(&[refspec], Some(&mut options))?;

    set_upstream_branch(repo_path, "origin", "master")?;

    Ok(())
}
