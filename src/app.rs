use crate::components::branchlist::BranchComponent;
use crate::components::commit_popup::CommitPopup;
use crate::components::diff::DiffComponent;
use crate::components::files::FileComponent;
use crate::components::log::LogComponent;
use crate::components::status::StatusComponent;
use crate::components::{Component, ComponentType};
use crate::Event;

use anyhow::Result;
use crossterm::event::KeyEvent;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub struct App {
    pub repo_path: PathBuf,
    pub branches: BranchComponent,
    pub logs: LogComponent,
    pub files: FileComponent,
    pub diff: DiffComponent,
    pub status: StatusComponent,
    pub commit_popup: CommitPopup,
    pub focused_component: ComponentType,
    pub event_sender: Sender<ComponentType>,
}

impl App {
    pub fn new(repo_path: PathBuf, event_sender: &Sender<ComponentType>) -> Self {
        Self {
            branches: BranchComponent::new(),
            logs: LogComponent::new(repo_path.clone()),
            files: FileComponent::new(repo_path.clone(), event_sender.clone()),
            diff: DiffComponent::new(repo_path.clone()),
            status: StatusComponent::new(repo_path.clone()),
            commit_popup: CommitPopup::new(repo_path.clone(), event_sender.clone()),
            focused_component: ComponentType::None,
            event_sender: event_sender.clone(),
            repo_path,
        }
    }

    pub fn is_popup_visible(&self) -> bool {
        self.commit_popup.visible()
    }

    pub fn update(&mut self) -> Result<()> {
        self.branches.update()?;
        self.diff.update()?;
        self.logs.update()?;
        self.status.update()?;
        self.files.update()?;
        Ok(())
    }

    pub fn handle_popup_event(&mut self, ev: Event<KeyEvent>) -> Result<()> {
        match ev {
            Event::Input(input) => {
                self.commit_popup.handle_event(input)?;
            }
            Event::Tick => {}
        }
        Ok(())
    }

    pub fn focus(&mut self, component: ComponentType) {
        let current_focus = self.focused_component.clone();
        self._focus(current_focus, false);
        self._focus(component, true);
    }

    fn _focus(&mut self, component: ComponentType, focus: bool) {
        match component {
            ComponentType::LogComponent => {
                self.logs.focus(focus);
            }
            ComponentType::DiffComponent => {
                self.diff.focus(focus);
            }
            ComponentType::BranchComponent => {
                self.branches.focus(focus);
            }
            ComponentType::FilesComponent => {
                self.files.focus(focus);
            }
            ComponentType::CommitPopup => {
                self.commit_popup.focus(focus);
            }
            ComponentType::None => {}
        }

        self.focused_component = component;
    }
}
