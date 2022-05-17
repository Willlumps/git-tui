use anyhow::Result;
use git2::{Config, Signature};
use std::path::PathBuf;

use crate::git::repo;

pub fn commit(repo_path: &PathBuf, message: &str) -> Result<()> {
    // TODO: Make this work for an initial commit
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
