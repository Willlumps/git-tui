use crate::components::branch_popup::BranchPopup;
use crate::components::branches::BranchComponent;
use crate::components::commit_popup::CommitPopup;
use crate::components::diff::DiffComponent;
use crate::components::error_popup::ErrorComponent;
use crate::components::files::FileComponent;
use crate::components::log::LogComponent;
use crate::components::push_popup::PushPopup;
use crate::components::status::StatusComponent;
use crate::components::{Component, ComponentType};
use crate::error::Error;
use crate::Event;

use std::path::PathBuf;

use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::KeyEvent;

pub enum ProgramEvent {
    Git(GitEvent),
    Focus(ComponentType),
    Error(Error),
}

pub enum GitEvent {
    PushSuccess,
    RefreshCommitLog,
    RefreshBranchList,
}

pub struct App {
    pub repo_path: PathBuf,
    pub branches: BranchComponent,
    pub logs: LogComponent,
    pub files: FileComponent,
    pub error_popup: ErrorComponent,
    pub diff: DiffComponent,
    pub status: StatusComponent,
    pub commit_popup: CommitPopup,
    pub push_popup: PushPopup,
    pub branch_popup: BranchPopup,
    pub focused_component: ComponentType,
    pub event_sender: Sender<ProgramEvent>,
}

impl App {
    pub fn new(repo_path: PathBuf, event_sender: &Sender<ProgramEvent>) -> App {
        Self {
            branches: BranchComponent::new(repo_path.clone(), event_sender.clone()),
            logs: LogComponent::new(repo_path.clone()),
            files: FileComponent::new(repo_path.clone(), event_sender.clone()),
            error_popup: ErrorComponent::new(event_sender.clone()),
            diff: DiffComponent::new(repo_path.clone()),
            status: StatusComponent::new(repo_path.clone()),
            commit_popup: CommitPopup::new(repo_path.clone(), event_sender.clone()),
            push_popup: PushPopup::new(),
            branch_popup: BranchPopup::new(repo_path.clone(), event_sender.clone()),
            focused_component: ComponentType::None,
            event_sender: event_sender.clone(),
            repo_path,
        }
    }

    pub fn is_popup_visible(&self) -> bool {
        self.commit_popup.visible()
            || self.push_popup.visible()
            || self.error_popup.visible()
            || self.branch_popup.visible()
    }

    pub fn update(&mut self) -> Result<(), Error> {
        self.branches.update()?;
        self.diff.update()?;
        self.logs.update()?;
        self.status.update()?;
        self.files.update()?;
        Ok(())
    }

    // TODO: I don't like how I did this, clean it up later
    pub fn handle_popup_input(&mut self, ev: Event<KeyEvent>) {
        match ev {
            Event::Input(input) => {
                if let Err(err) = self._handle_popup_input(input) {
                    self.event_sender
                        .send(ProgramEvent::Error(err))
                        .expect("Send Failed");
                }
            }
            Event::Tick => {}
        }
    }

    fn _handle_popup_input(&mut self, ev: KeyEvent) -> Result<(), Error> {
        self.commit_popup.handle_event(ev)?;
        self.push_popup.handle_event(ev)?;
        self.error_popup.handle_event(ev)?;
        self.branch_popup.handle_event(ev)?;
        Ok(())
    }

    pub fn handle_input(&mut self, ev: KeyEvent) {
        if let Err(err) = self._handle_input(ev) {
            self.event_sender
                .send(ProgramEvent::Error(err))
                .expect("Send Failed");
        }
    }

    fn _handle_input(&mut self, ev: KeyEvent) -> Result<(), Error> {
        self.branches.handle_event(ev)?;
        self.logs.handle_event(ev)?;
        self.diff.handle_event(ev)?;
        self.files.handle_event(ev)?;
        Ok(())
    }

    pub fn handle_git_event(&mut self, ev: GitEvent) -> Result<(), Error> {
        match ev {
            GitEvent::PushSuccess => {
                self.push_popup.set_message("Push Successfull!");
            }
            GitEvent::RefreshCommitLog => {
                self.logs.update()?;
            }
            GitEvent::RefreshBranchList => {
                self.branches.update()?;
            }
        }
        Ok(())
    }

    pub fn display_error(&mut self, error: Error) {
        match error {
            Error::Git(err) => {
                self.error_popup.set_git_error(err);
            }
            Error::Unknown(message) => {
                self.error_popup.set_message(message);
            }
            _ => {}
        }
        self.focus(ComponentType::ErrorComponent);
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
            ComponentType::ErrorComponent => {
                self.error_popup.focus(focus);
            }
            ComponentType::BranchComponent => {
                self.branches.focus(focus);
            }
            ComponentType::FilesComponent => {
                self.files.focus(focus);
            }
            ComponentType::CommitComponent => {
                self.commit_popup.focus(focus);
            }
            ComponentType::PushComponent => {
                self.push_popup.focus(focus);
            }
            ComponentType::BranchPopupComponent => {
                self.branch_popup.focus(focus);
            }
            ComponentType::None => {}
        }

        self.focused_component = component;
    }
}
