use crate::components::branch_popup::BranchPopup;
use crate::components::log_popup::LogPopup;
use crate::components::branches::BranchComponent;
use crate::components::commit_popup::CommitPopup;
use crate::components::diff::DiffComponent;
use crate::components::error_popup::ErrorComponent;
use crate::components::files::FileComponent;
use crate::components::log::LogComponent;
use crate::components::message_popup::MessagePopup;
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
    FetchSuccess,
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
    pub branch_popup: BranchPopup,
    pub message_popup: MessagePopup,
    pub log_popup: LogPopup,
    pub focused_component: ComponentType,
    pub event_sender: Sender<ProgramEvent>,
}

impl App {
    pub fn new(repo_path: PathBuf, event_sender: &Sender<ProgramEvent>) -> App {
        Self {
            branches: BranchComponent::new(repo_path.clone(), event_sender.clone()),
            logs: LogComponent::new(repo_path.clone(), event_sender.clone()),
            files: FileComponent::new(repo_path.clone(), event_sender.clone()),
            error_popup: ErrorComponent::new(event_sender.clone()),
            diff: DiffComponent::new(repo_path.clone()),
            status: StatusComponent::new(repo_path.clone()),
            commit_popup: CommitPopup::new(repo_path.clone(), event_sender.clone()),
            branch_popup: BranchPopup::new(repo_path.clone(), event_sender.clone()),
            message_popup: MessagePopup::new(),
            log_popup: LogPopup::new(event_sender.clone()),
            focused_component: ComponentType::None,
            event_sender: event_sender.clone(),
            repo_path,
        }
    }

    pub fn is_popup_visible(&self) -> bool {
        self.commit_popup.visible()
            || self.error_popup.visible()
            || self.branch_popup.visible()
            || self.message_popup.visible()
            || self.log_popup.visible()
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
        // TODO: Single out which popup to send events to.
        //       It will currently try to focus multiple components and cause
        //       UI glitches
        self.commit_popup.handle_event(ev)?;
        self.error_popup.handle_event(ev)?;
        self.branch_popup.handle_event(ev)?;
        self.message_popup.handle_event(ev)?;
        self.log_popup.handle_event(ev)?;
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
                self.message_popup.set_message("Push Successfull!");
            }
            GitEvent::FetchSuccess => {
                self.message_popup.set_message("Fetch Successfull!");
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
        match component.clone() {
            ComponentType::LogComponent => {
                self.logs.focus(focus);
            }
            ComponentType::FullLogComponent(commit) => {
                self.log_popup.set_commit(commit);
                self.log_popup.focus(focus);
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
            ComponentType::BranchPopupComponent => {
                self.branch_popup.focus(focus);
            }
            ComponentType::MessageComponent(message) => {
                self.message_popup.set_message(&message);
                self.message_popup.focus(focus);
            }
            ComponentType::None => {}
        }

        self.focused_component = component;
    }
}
