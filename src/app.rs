use crate::components::branchlist::BranchComponent;
use crate::components::diff::DiffComponent;
use crate::components::files::FileComponent;
use crate::components::log::LogComponent;
use crate::components::status::StatusComponent;

pub struct App<'src> {
    pub input: String,
    pub branches: BranchComponent,
    pub logs: LogComponent<'src>,
    pub files: FileComponent,
    pub diff: DiffComponent,
    pub status: StatusComponent,
    pub repo_path: &'src str,
}

impl<'src> App<'src> {
    pub fn new(repo_path: &'src str) -> Self {
        Self {
            input: String::new(),
            branches: BranchComponent::new(),
            logs: LogComponent::new(repo_path),
            files: FileComponent::new(),
            diff: DiffComponent::new(repo_path),
            status: StatusComponent::new(),
            repo_path,
        }
    }
}
