use anyhow::Result;
use crossterm::event::KeyEvent;
use tui::layout::{Constraint, Direction, Layout, Rect};

pub mod branchlist;
pub mod commit_popup;
pub mod diff;
pub mod files;
pub mod log;
pub mod status;

#[derive(Clone, Debug)]
pub enum ComponentType {
    None,
    LogComponent,
    DiffComponent,
    FilesComponent,
    BranchComponent,
    CommitPopup,
}

pub trait Component {
    fn update(&mut self) -> Result<()>;
    fn focus(&mut self, focus: bool);
    fn handle_event(&mut self, ev: KeyEvent) -> Result<()>;
}

fn centered_rect(percent_x: u16, y: u16, r: Rect) -> Rect {
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
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
