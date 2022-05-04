use crate::components::branchlist::BranchComponent;

use tui::style::{Color, Style};

pub struct App {
    //repo: &'a Repository,
    pub input: String,
    pub branches: BranchComponent,
    pub style: Style,
}

impl App {
    //fn new(repo: &'a Repository) -> Self {
    pub fn new() -> Self {
        Self {
            //repo,
            input: String::new(),
            branches: BranchComponent::new(),
            style: Style::default().fg(Color::White),
        }
    }
}
