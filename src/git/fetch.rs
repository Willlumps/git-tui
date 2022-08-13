use crate::error::Error;
use crate::git::diff::head;
use crate::git::repo;

use std::path::Path;

use crossbeam::channel::Sender;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};

pub fn pull_head(repo_path: &Path, _progress_sender: Sender<bool>) -> Result<(), Error> {
    let head = head(repo_path)?;
    fetch(repo_path, _progress_sender)?;
    merge(repo_path, &head)?;
    Ok(())
}

pub fn pull_selected(
    repo_path: &Path,
    branch_name: &str,
    _progress_sender: Sender<bool>,
) -> Result<(), Error> {
    fetch(repo_path, _progress_sender)?;
    merge(repo_path, branch_name)?;
    Ok(())
}

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

// Source: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
fn merge(repo_path: &Path, branch_name: &str) -> Result<(), Error> {
    let repo = repo(repo_path)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let annotated_commit = repo.reference_to_annotated_commit(&fetch_head)?;

    let (analysis, preference) = repo.merge_analysis(&[&annotated_commit])?;

    if analysis.is_fast_forward() {
        if preference.is_no_fast_forward() {
            return Err(Error::Git(git2::Error::from_str(
                "Fast forward merges are not allowed",
            )));
        }
        let refname = format!("refs/heads/{}", branch_name);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                ff_merge(&repo, &mut r, &annotated_commit)?;
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    annotated_commit.id(),
                    true,
                    &format!("Setting {} to {}", branch_name, annotated_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.is_normal() {
        normal_merge()?;
    }

    Ok(())
}

fn ff_merge(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };

    lb.set_target(rc.id(), "")?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    Ok(())
}

fn normal_merge() -> Result<(), Error> {
    Err(Error::Unknown("Normal merge unimplemented".to_string()))
}
