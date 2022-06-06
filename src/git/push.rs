use crate::error::Error;
use crate::git::diff::head;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use crossbeam::channel::Sender;
use git2::{Cred, PushOptions, RemoteCallbacks};

pub fn push(repo_path: &Path, progress_sender: Sender<usize>) -> Result<(), Error> {
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

    callbacks.push_transfer_progress(|current, total, _bytes| {
        let percentage = (current / total) * 100;
        progress_sender.send(percentage).expect("Send failed");
    });

    callbacks.push_update_reference(|_remote, _status| {
        // TODO
        Ok(())
    });

    let mut options = PushOptions::new();
    let head = head(repo_path)?;
    let refspec = format!("refs/heads/{}", head);

    options.remote_callbacks(callbacks);
    remote.push(&[refspec], Some(&mut options))?;

    Ok(())
}
