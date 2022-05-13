use git2::Repository;
use std::error::Error;

pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl DiffStats {
    pub fn get_stats(repo_path: &str) -> Result<DiffStats, Box<dyn Error>> {
        let repo = match Repository::init(repo_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to init: {}", e),
        };

        let mut opt = git2::DiffOptions::new();
        let diff = repo.diff_index_to_workdir(None, Some(&mut opt))?;
        let stats = diff.stats()?;

        Ok(DiffStats {
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }
}

