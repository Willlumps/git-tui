use super::repo;
use anyhow::Result;
use std::path::Path;

pub struct Branch {
    // This will probably need more fields, though as of now I don't know
    // what those would be.
    pub name: String,
}

pub fn branches(repo_path: &Path) -> Result<Vec<Branch>> {
    let repo = repo(repo_path)?;
    let git_branches = repo.branches(Some(git2::BranchType::Local))?;
    let mut branches = Vec::new();

    for git_branch in git_branches {
        let branch = git_branch?.0;
        if let Some(name) = branch.name()? {
            branches.push(Branch {
                name: name.to_string(),
            });
        }
    }
    Ok(branches)
}
