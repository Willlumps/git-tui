pub mod branch_popup;
pub mod branches;
pub mod cherry_pick_popup;
pub mod commit_popup;
pub mod diff;
pub mod diff_staged;
pub mod error_popup;
pub mod files;
pub mod log;
pub mod log_popup;
pub mod message_popup;
pub mod status;

use crate::error::Error;
use crate::git::log::Commit;

use anyhow::Result;
use crossterm::event::KeyEvent;
use tui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Clone, Debug)]
pub enum ComponentType {
    None,
    BranchComponent,
    BranchPopupComponent,
    CherryPickPopup(Vec<Commit>),
    CommitComponent,
    DiffComponent,
    DiffStagedComponent,
    ErrorComponent,
    FilesComponent,
    LogComponent,
    FullLogComponent(Commit),
    MessageComponent(String),
}

pub trait Component {
    fn update(&mut self) -> Result<(), Error>;
    fn focus(&mut self, focus: bool);
    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error>;
}

pub fn centered_rect(x: u16, y: u16, r: Rect) -> Rect {
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
