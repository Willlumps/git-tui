use crate::components::branchlist::BranchComponent;
use crate::components::diff::DiffComponent;
use crate::components::files::FileComponent;
use crate::components::log::LogComponent;
use crate::components::status::StatusComponent;

use std::path::PathBuf;

pub struct App {
    pub branches: BranchComponent,
    pub logs: LogComponent,
    pub files: FileComponent,
    pub diff: DiffComponent,
    pub status: StatusComponent,
    pub repo_path: PathBuf,
}

impl App {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            branches: BranchComponent::new(),
            logs: LogComponent::new(repo_path.clone()),
            files: FileComponent::new(),
            diff: DiffComponent::new(repo_path.clone()),
            status: StatusComponent::new(repo_path.clone()),
            repo_path,
        }
    }
}
