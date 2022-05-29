use crate::error::Error;
use crate::git::diff::head;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use crossbeam::channel::Sender;
use git2::{Cred, PushOptions, RemoteCallbacks};

pub fn push(repo_path: &Path, progress_sender: Sender<bool>) -> Result<(), Error> {
    let repo = repo(repo_path)?;

    let mut remote = repo.find_remote("origin")?;
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
    callbacks.push_transfer_progress(|_current, _total, _bytes| {
        // TODO: Progress bar in the future?
        progress_sender
            .send(true)
            .expect("Push progress send failed.");
    });
    callbacks.push_update_reference(|_remote, status| {
        if let Some(message) = status {
            return Err(git2::Error::from_str(message));
        }
        Ok(())
    });

    let mut options = PushOptions::new();
    let head = head(repo_path)?;
    let refspec = format!("refs/heads/{}", head);

    options.remote_callbacks(callbacks);
    remote.push(&[refspec], Some(&mut options))?;

    progress_sender
        .send(false)
        .expect("Push progress send failed.");
    Ok(())
}
