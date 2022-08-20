pub mod branch_popup;
pub mod branches;
pub mod cherry_pick_popup;
pub mod commit_popup;
pub mod diff;
pub mod error_popup;
pub mod files;
pub mod log;
pub mod log_popup;
pub mod message_popup;
pub mod remote_popup;
pub mod status;

use crate::error::Error;
use crate::git::diff::DiffComponentType;
use crate::git::log::Commit;

use anyhow::Result;
use crossterm::event::KeyEvent;
use tui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Clone, Debug)]
pub enum ComponentType {
    BranchComponent,
    BranchPopupComponent,
    CommitComponent,
    ErrorComponent,
    FilesComponent,
    LogComponent,
    RemotePopupComponent,
    CherryPickPopup(Vec<Commit>),
    DiffComponent(DiffComponentType),
    FullLogComponent(Commit),
    MessageComponent(String),
    None,
}

pub trait Component {
    fn update(&mut self) -> Result<(), Error>;
    fn focus(&mut self, focus: bool);
    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error>;
}

pub trait ScrollableComponent {
    fn scroll_up(&mut self, amount: usize) {
        let position = self.get_position().saturating_sub(amount);
        self.set_position(position);
        self.set_state(position)
    }

    fn scroll_down(&mut self, amount: usize) {
        let len = self.get_list_length();
        let position = max(self.get_position() + amount, len - 1);
        self.set_position(position);
        self.set_state(position);
    }

    fn reset_state(&mut self) {
        self.set_position(0);
        self.set_state(0);
    }

    fn get_list_length(&self) -> usize;
    fn get_position(&self) -> usize;
    fn set_position(&mut self, position: usize);
    fn set_state(&mut self, position: usize);
}

fn max(amount: usize, max: usize) -> usize {
    if amount > max {
        max
    } else {
        amount
    }
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
