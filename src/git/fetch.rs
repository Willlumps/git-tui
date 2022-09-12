use crate::git::diff::head;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use crossbeam::channel::Sender;
use git2::{FetchOptions, Repository};

use super::{callbacks::create_remote_callbacks, remote::get_remote};

pub fn pull_head(repo_path: &Path, _progress_sender: Sender<usize>) -> Result<()> {
    let head = head(repo_path)?;
    fetch(repo_path, _progress_sender)?;
    merge(repo_path, &head)?;
    Ok(())
}

pub fn pull_selected(
    repo_path: &Path,
    branch_name: &str,
    _progress_sender: Sender<usize>,
) -> Result<()> {
    fetch(repo_path, _progress_sender)?;
    merge(repo_path, branch_name)?;
    Ok(())
}

pub fn fetch(repo_path: &Path, _progress_sender: Sender<usize>) -> Result<()> {
    // TODO: Fetch from all/multiple remotes if available
    let repo = repo(repo_path)?;

    let remote = match get_remote(repo_path)? {
        Some(remote_name) => remote_name,
        None => {
            return Err(anyhow::Error::msg("Fetch: no remotes found"));
        }
    };

    let callbacks = create_remote_callbacks(_progress_sender, None);

    let mut options = FetchOptions::new();
    options.download_tags(git2::AutotagOption::All);
    options.remote_callbacks(callbacks);

    repo.find_remote(&remote)?
        .fetch(&[] as &[&str], Some(&mut options), None)?;

    Ok(())
}

// Source: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
fn merge(repo_path: &Path, branch_name: &str) -> Result<()> {
    let repo = repo(repo_path)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let annotated_commit = repo.reference_to_annotated_commit(&fetch_head)?;

    let (analysis, preference) = repo.merge_analysis(&[&annotated_commit])?;

    if analysis.is_fast_forward() {
        if preference.is_no_fast_forward() {
            return Err(anyhow::Error::msg("Fast forward merges are not allowed"));
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
) -> Result<()> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };

    lb.set_target(rc.id(), "")?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    Ok(())
}

fn normal_merge() -> Result<()> {
    Err(anyhow::Error::msg("Normal merge unimplemented"))
}
