use crate::error::Error;
use crate::git::diff::head;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use crossbeam::channel::Sender;
use git2::{Cred, PushOptions, RemoteCallbacks};

pub fn push(repo_path: &Path, progress_sender: Sender<i8>) -> Result<(), Error> {
    let repo = repo(repo_path)?;

    let mut first_credentials_cb = true;
    let mut callbacks = RemoteCallbacks::new();
    let mut remote = repo.find_remote("origin")?;

    callbacks.credentials(|_url, username_from_url, allowed_types| {
        let cred: Result<Cred, git2::Error>;
        if !first_credentials_cb {
            progress_sender.send(-1).expect("Send failed");
        }
        if allowed_types.is_ssh_key() {
            cred = match username_from_url {
                Some(username) => Cred::ssh_key_from_agent(username),
                None => Err(git2::Error::from_str("Where da username??")),
            }
        } else if allowed_types.is_user_pass_plaintext() {
            // Do people actually use plaintext user/pass ??
            unimplemented!();
        } else {
            cred = Cred::default();
        }
        first_credentials_cb = false;
        cred
    });

    callbacks.push_transfer_progress(|current, total, _bytes| {
        if let Some(percentage) = current.checked_div(total) {
            progress_sender.send((percentage * 100) as i8).expect("Send failed");
        } else {
            progress_sender.send(100).expect("Send failed");
        }
    });

    callbacks.push_update_reference(|_remote, _status| {
        // TODO
        if _status.is_some() {
            panic!("oh no {}", _status.unwrap());
        }
        Ok(())
    });

    let mut options = PushOptions::new();
    let head = head(repo_path)?;
    let refspec = format!("refs/heads/{}", head);

    options.remote_callbacks(callbacks);
    remote.push(&[refspec], Some(&mut options))?;

    Ok(())
}
