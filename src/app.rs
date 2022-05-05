use crate::components::branchlist::BranchComponent;
use crate::components::log::LogComponent;

use tui::style::{Color, Style};

pub struct App {
    //repo: &'a Repository,
    pub input: String,
    pub branches: BranchComponent,
    pub logs: LogComponent,
}

impl App {
    //fn new(repo: &'a Repository) -> Self {
    pub fn new() -> Self {
        Self {
            //repo,
            input: String::new(),
            branches: BranchComponent::new(),
            logs: LogComponent::new(),
        }
    }
}
