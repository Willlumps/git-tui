use anyhow::Result;
use git2::{Cred, PushOptions, RemoteCallbacks};
use std::path::Path;
use std::env;
use super::repo;

pub fn push(repo_path: &Path) -> Result<()>{
    let repo = repo(repo_path)?;

    let mut remote = repo.find_remote("origin")?;
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
            None,
        )
    });
    callbacks.push_transfer_progress(|_current, _total, _bytes| {
        // TODO: Progress bar in the future?
    });
    callbacks.push_update_reference(|_remote, status| {
        if status.is_some() {
            // Push failed, log something eventually
        }
        Ok(())
    });

    let mut options = PushOptions::new();
    options.remote_callbacks(callbacks);

    remote.push(&["refs/heads/master"], Some(&mut options))?;
    Ok(())
}
