use std::path::PathBuf;

use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::Frame;

use crate::components::branch_popup::BranchPopup;
use crate::components::branches::BranchComponent;
use crate::components::cherry_pick_popup::CherryPickPopup;
use crate::components::commit_popup::CommitPopup;
use crate::components::diff::DiffComponent;
use crate::components::error_popup::ErrorComponent;
use crate::components::files::FileComponent;
use crate::components::log::LogComponent;
use crate::components::log_popup::LogPopup;
use crate::components::message_popup::MessagePopup;
use crate::components::remote_popup::RemotePopupComponent;
use crate::components::status::StatusComponent;
use crate::components::{Component, ComponentType};
use crate::error::Error;
use crate::git::diff::DiffComponentType;
use crate::Event;

pub enum ProgramEvent {
    Exit,
    Error(Error),
    Focus(ComponentType),
    Git(GitEvent),
}

#[allow(dead_code)]
pub enum GitEvent {
    FetchSuccess,
    PushSuccess,
    RefreshCommitLog,
    RefreshBranchList,
}

pub struct App {
    pub branches: BranchComponent,
    pub branch_popup: BranchPopup,
    pub cherry_pick_popup: CherryPickPopup,
    pub commit_popup: CommitPopup,
    pub diff: DiffComponent,
    pub diff_staged: DiffComponent,
    pub error_popup: ErrorComponent,
    pub event_sender: Sender<ProgramEvent>,
    pub files: FileComponent,
    pub focused_component: ComponentType,
    pub logs: LogComponent,
    pub log_popup: LogPopup,
    pub message_popup: MessagePopup,
    pub remote_popup: RemotePopupComponent,
    pub status: StatusComponent,
    pub repo_path: PathBuf,
}

impl App {
    pub fn new(repo_path: PathBuf, event_sender: &Sender<ProgramEvent>) -> Self {
        Self {
            branches: BranchComponent::new(repo_path.clone(), event_sender.clone()),
            branch_popup: BranchPopup::new(repo_path.clone(), event_sender.clone()),
            cherry_pick_popup: CherryPickPopup::new(repo_path.clone(), event_sender.clone()),
            commit_popup: CommitPopup::new(repo_path.clone(), event_sender.clone()),
            diff: DiffComponent::new(repo_path.clone(), DiffComponentType::Diff),
            diff_staged: DiffComponent::new(repo_path.clone(), DiffComponentType::Staged),
            error_popup: ErrorComponent::new(event_sender.clone()),
            event_sender: event_sender.clone(),
            files: FileComponent::new(repo_path.clone(), event_sender.clone()),
            focused_component: ComponentType::None,
            logs: LogComponent::new(repo_path.clone(), event_sender.clone()),
            log_popup: LogPopup::new(event_sender.clone()),
            message_popup: MessagePopup::new(),
            remote_popup: RemotePopupComponent::new(repo_path.clone(), event_sender.clone()),
            status: StatusComponent::new(repo_path.clone()),
            repo_path,
        }
    }

    pub fn is_popup_visible(&self) -> bool {
        self.commit_popup.visible()
            || self.cherry_pick_popup.visible()
            || self.error_popup.visible()
            || self.branch_popup.visible()
            || self.message_popup.visible()
            || self.log_popup.visible()
            || self.remote_popup.visible()
    }

    pub fn draw_popup<B: Backend>(&mut self, f: &mut Frame<B>, size: Rect) -> Result<()> {
        match self.focused_component {
            ComponentType::BranchPopupComponent => self.branch_popup.draw(f, size)?,
            ComponentType::CommitComponent => self.commit_popup.draw(f, size)?,
            ComponentType::ErrorComponent => self.error_popup.draw(f, size)?,
            ComponentType::RemotePopupComponent => self.remote_popup.draw(f, size),
            ComponentType::CherryPickPopup(_) => self.cherry_pick_popup.draw(f, size)?,
            ComponentType::FullLogComponent(_) => self.log_popup.draw(f, size)?,
            ComponentType::MessageComponent(_) => self.message_popup.draw(f, size)?,
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), Error> {
        self.branches.update()?;
        self.diff.update()?;
        self.diff_staged.update()?;
        self.logs.update()?;
        self.status.update()?;
        self.files.update()?;
        Ok(())
    }

    pub fn handle_input_event(&mut self, ev: Event<KeyEvent>) -> Result<(), Error> {
        match ev {
            Event::Input(input) => match input.code {
                KeyCode::Char('1') => self.focus(ComponentType::FilesComponent),
                KeyCode::Char('2') => self.focus(ComponentType::BranchComponent),
                KeyCode::Char('3') => self.focus(ComponentType::LogComponent),
                KeyCode::Char('4') => {
                    self.focus(ComponentType::DiffComponent(DiffComponentType::Diff))
                }
                KeyCode::Char('5') => {
                    self.focus(ComponentType::DiffComponent(DiffComponentType::Staged))
                }
                KeyCode::Esc if !self.is_popup_visible() => self
                    .event_sender
                    .send(ProgramEvent::Exit)
                    .expect("Send failed"),
                _ => self.handle_input(input)?,
            },
            Event::Tick => {}
        }

        Ok(())
    }

    pub fn handle_input(&mut self, ev: KeyEvent) -> Result<(), Error> {
        match &mut self.focused_component {
            ComponentType::LogComponent => self.logs.handle_event(ev)?,
            ComponentType::ErrorComponent => self.error_popup.handle_event(ev)?,
            ComponentType::BranchComponent => self.branches.handle_event(ev)?,
            ComponentType::FilesComponent => self.files.handle_event(ev)?,
            ComponentType::CommitComponent => self.commit_popup.handle_event(ev)?,
            ComponentType::BranchPopupComponent => self.branch_popup.handle_event(ev)?,
            ComponentType::RemotePopupComponent => self.remote_popup.handle_event(ev)?,
            ComponentType::CherryPickPopup(_) => self.cherry_pick_popup.handle_event(ev)?,
            ComponentType::MessageComponent(_) => self.message_popup.handle_event(ev)?,
            ComponentType::FullLogComponent(_) => self.log_popup.handle_event(ev)?,
            ComponentType::DiffComponent(diff_type) => match diff_type {
                DiffComponentType::Diff => self.diff.handle_event(ev)?,
                DiffComponentType::Staged => self.diff_staged.handle_event(ev)?,
            },
            ComponentType::None => {}
        }

        Ok(())
    }

    pub fn handle_git_event(&mut self, ev: GitEvent) -> Result<(), Error> {
        match ev {
            GitEvent::PushSuccess => self.message_popup.set_message("Push Successfull!"),
            GitEvent::FetchSuccess => self.message_popup.set_message("Fetch Successfull!"),
            GitEvent::RefreshCommitLog => self.logs.update()?,
            GitEvent::RefreshBranchList => self.branches.update()?,
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
            ComponentType::LogComponent => self.logs.focus(focus),
            ComponentType::ErrorComponent => self.error_popup.focus(focus),
            ComponentType::BranchComponent => self.branches.focus(focus),
            ComponentType::FilesComponent => self.files.focus(focus),
            ComponentType::CommitComponent => self.commit_popup.focus(focus),
            ComponentType::BranchPopupComponent => self.branch_popup.focus(focus),
            ComponentType::RemotePopupComponent => self.remote_popup.focus(focus),
            ComponentType::DiffComponent(diff_type) => match diff_type {
                DiffComponentType::Diff => self.diff.focus(focus),
                DiffComponentType::Staged => self.diff_staged.focus(focus),
            },
            ComponentType::CherryPickPopup(logs) => {
                self.cherry_pick_popup.set_logs(logs);
                self.cherry_pick_popup.focus(focus);
            }
            ComponentType::MessageComponent(message) => {
                self.message_popup.set_message(&message);
                self.message_popup.focus(focus);
            }
            ComponentType::FullLogComponent(commit) => {
                self.log_popup.set_commit(commit);
                self.log_popup.focus(focus);
            }
            ComponentType::None => {}
        }

        self.focused_component = component;
    }
}
