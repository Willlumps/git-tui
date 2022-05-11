use crate::components::branchlist::BranchComponent;
use crate::components::diff::DiffComponent;
use crate::components::files::FileComponent;
use crate::components::log::LogComponent;
use crate::components::status::StatusComponent;

pub struct App {
    pub input: String,
    pub branches: BranchComponent,
    pub logs: LogComponent,
    pub files: FileComponent,
    pub diff: DiffComponent,
    pub status: StatusComponent,
    pub repo_path: String,
}

impl App {
    pub fn new(repo_path: String) -> Self {
        Self {
            //repo,
            input: String::new(),
            branches: BranchComponent::new(),
            logs: LogComponent::new(repo_path.as_str()),
            files: FileComponent::new(),
            diff: DiffComponent::new(repo_path.as_str()),
            status: StatusComponent::new(),
            repo_path,
        }
    }
}
