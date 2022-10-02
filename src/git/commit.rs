use std::path::Path;

use anyhow::Result;
use git2::{AnnotatedCommit, Config, Oid, Signature};

use crate::git::log::Commit;
use crate::git::repo;

pub fn create_initial_commit(repo_path: &Path) -> Result<()> {
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

pub fn commit(repo_path: &Path, message: &str, merge_commit: Option<git2::Oid>) -> Result<()> {
    let repo = repo(repo_path)?;
    let signature = signature()?;

    let mut index = repo.index()?;
    let id = index.write_tree()?;
    let tree = repo.find_tree(id)?;

    let mut merge_commit_id = match merge_commit {
        Some(id) => vec![repo.find_commit(id)?],
        None => Vec::new(),
    };

    if let Some(head) = repo.head()?.target() {
        let mut commit = vec![repo.find_commit(head)?];
        commit.append(&mut merge_commit_id);

        let parents = commit.iter().collect::<Vec<_>>();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            parents.as_slice(),
        )?;
    } else {
        return Err(anyhow::Error::msg("Make me a meaninful error message"));
    }

    Ok(())
}

pub fn merge_commit(repo_path: &Path, annotated_commit: AnnotatedCommit) -> Result<()> {
    // I'm not certain the 'merge' will fail if there is a conflict since
    // we have to manually commit anyways. I don't want to test it so I'll assume
    // I have to check it here before attempting to commit :D
    let repo = repo(repo_path)?;
    let index = repo.index()?;

    if index.has_conflicts() {
        return Err(anyhow::anyhow!(
            "Conflicts or something, you should fix them and then commit."
        ));
    } else {
        let msg = repo.message()?;
        commit(repo_path, &msg, Some(annotated_commit.id()))?;
    }

    Ok(())
}

pub fn revert_commit(repo_path: &Path, commit: &Commit) -> Result<()> {
    let repo = repo(repo_path)?;
    let oid = Oid::from_str(commit.id())?;
    let commit = repo.find_commit(oid)?;

    repo.revert(&commit, None)?;

    Ok(())
}

pub fn cherry_pick(repo_path: &Path, oids: &Vec<String>) -> Result<()> {
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

fn signature() -> Result<Signature<'static>> {
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
