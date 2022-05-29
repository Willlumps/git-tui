pub mod branch_popup;
pub mod branches;
pub mod commit_popup;
pub mod diff;
pub mod error_popup;
pub mod files;
pub mod log;
pub mod push_popup;
pub mod status;

use crate::error::Error;

use anyhow::Result;
use crossterm::event::KeyEvent;
use tui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Clone, Debug)]
pub enum ComponentType {
    None,
    LogComponent,
    DiffComponent,
    ErrorComponent,
    FilesComponent,
    BranchComponent,
    BranchPopupComponent,
    CommitComponent,
    PushComponent,
}

pub trait Component {
    fn update(&mut self) -> Result<(), Error>;
    fn focus(&mut self, focus: bool);
    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error>;
}

fn centered_rect(x: u16, y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((r.height - y) / 2),
                Constraint::Length(y),
                Constraint::Length((r.height - y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - x) / 2),
                Constraint::Length(x),
                Constraint::Length((r.width - x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
