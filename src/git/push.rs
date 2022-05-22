use crate::app::{GitEvent, ProgramEvent};
use anyhow::Result;
use git2::{Cred, PushOptions, RemoteCallbacks};
use std::path::Path;
use std::sync::mpsc::Sender;

use super::git_diff::head;
use super::repo;

pub fn push(
    repo_path: &Path,
    progress_sender: Sender<bool>,
    event_sender: Sender<ProgramEvent>,
) -> Result<()> {
    let repo = repo(repo_path)?;

    let mut remote = repo.find_remote("origin")?;
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|_url, username_from_url, allowed_types| {
        if allowed_types.is_ssh_key() {
            match username_from_url {
                Some(username) => {
                    Cred::ssh_key_from_agent(username)
                }
                None => {
                    Err(git2::Error::from_str("Where da username??"))
                }
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
        if let Err(err) = progress_sender.send(true) {
            eprintln!("Progress send failed: {err}");
        }
    });
    callbacks.push_update_reference(|_remote, status| {
        if let Some(message) = status {
            if let Err(err) = event_sender.send(ProgramEvent::GitEvent(GitEvent::PushFailure(
                message.to_string(),
            ))) {
                eprintln!("Event send failure: {err}");
            }
        }
        Ok(())
    });

    let mut options = PushOptions::new();
    let head = head(repo_path)?;
    let refspec = format!("refs/heads/{}", head);

    options.remote_callbacks(callbacks);
    remote.push(&[refspec], Some(&mut options))?;

    progress_sender.send(false)?;
    Ok(())
}
