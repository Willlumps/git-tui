use git2::Repository;
use git2::StatusOptions;
use std::error::Error;

pub fn get_modified_files(repo_path: &str) -> Result<(), Box<dyn Error>> {
    let repo = match Repository::init(repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to init: {}", e),
    };

    let statuses = repo.statuses(Some(&mut StatusOptions::new()))?;
    println!("{}", statuses.len());

    if let Some(status) = statuses.get(0) {
        println!("{:?}", status.status());
        println!("{:?}", status.path().unwrap());
    }

    Ok(())
}
