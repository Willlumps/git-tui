use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod branchlist;
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
}

pub trait Component {
    fn update(&mut self) -> Result<()>;
    fn focus(&mut self, focus: bool);
    fn handle_event(&mut self, ev: KeyEvent) -> Result<()>;
}
