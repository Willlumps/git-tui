use crate::error::Error;
use crate::git::log::Commit;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use git2::{Config, Oid, Signature};

pub fn create_initial_commit(repo_path: &Path) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let signature = signature()?;

    let mut index = repo.index()?;
    let id = index.write_tree()?;
    let tree = repo.find_tree(id)?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )?;
    Ok(())
}

pub fn commit(repo_path: &Path, message: &str) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let signature = signature()?;

    let mut index = repo.index()?;
    let id = index.write_tree()?;
    let tree = repo.find_tree(id)?;

    if let Some(head) = repo.head()?.target() {
        let commit = vec![repo.find_commit(head)?];
        let parents = commit.iter().collect::<Vec<_>>();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            parents.as_slice(),
        )?;
    }
    Ok(())
}

pub fn revert_commit(repo_path: &Path, commit: &Commit) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let oid = Oid::from_str(commit.id())?;
    let commit = repo.find_commit(oid)?;

    repo.revert(&commit, None)?;

    Ok(())
}

pub fn cherry_pick(repo_path: &Path, oids: &Vec<String>) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    let head = repo.head()?.peel_to_commit()?;
    let opts = git2::MergeOptions::new();
    let commiter = signature()?;

    // Don't ask
    for oid in oids {
        let commit = repo.find_commit(Oid::from_str(oid)?)?;
        let mut index = repo.cherrypick_commit(&commit, &head, 0, Some(&opts))?;
        let tree_oid = index.write_tree_to(&repo)?;
        let tree = repo.find_tree(tree_oid)?;
        let author = commit.author();
        let message = commit.message().unwrap_or("");

        repo.commit(Some("HEAD"), &author, &commiter, message, &tree, &[&head])?;
    }

    Ok(())
}

fn signature() -> Result<Signature<'static>, Error> {
    // Is there a better way to do this?
    let config = Config::open_default()?;

    if let Some(name) = config.get_entry("user.name")?.value() {
        if let Some(email) = config.get_entry("user.email")?.value() {
            let signature = Signature::now(name, email)?;
            return Ok(signature);
        }

        let signature = Signature::now(name, "")?;
        return Ok(signature);
    }

    let signature = Signature::now("", "")?;

    Ok(signature)
}
