use crate::error::Error;
use crate::git::repo;

use std::path::Path;

use crossbeam::channel::Sender;
use git2::{Cred, FetchOptions, RemoteCallbacks};

pub fn fetch(repo_path: &Path, _progress_sender: Sender<bool>) -> Result<(), Error> {
    // TODO: Fetch from all/multiple remotes if available
    let repo = repo(repo_path)?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, allowed_types| {
        if allowed_types.is_ssh_key() {
            match username_from_url {
                Some(username) => Cred::ssh_key_from_agent(username),
                None => Err(git2::Error::from_str("Where da username??")),
            }
        } else if allowed_types.is_user_pass_plaintext() {
            // Do people actually use plaintext user/pass ??
            unimplemented!();
        } else {
            Cred::default()
        }
    });

    let mut options = FetchOptions::new();
    options.download_tags(git2::AutotagOption::All);
    options.remote_callbacks(callbacks);
    repo.find_remote("origin")?
        .fetch(&[] as &[&str], Some(&mut options), None)?;

    Ok(())
}
