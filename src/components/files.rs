use std::path::PathBuf;
use std::thread;

use anyhow::Result;
use core::time::Duration;
use crossbeam::channel::{unbounded, Sender};
use crossterm::event::{KeyCode, KeyEvent};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState};
use tui::Frame;

use crate::app::ProgramEvent;
use crate::component_style::ComponentTheme;
use crate::components::{Component, ComponentType, ScrollableComponent};
use crate::error::Error;
use crate::git::remote::{get_remotes, push};
use crate::git::stage::{stage_all, stage_file, unstage_all, unstage_file};
use crate::git::status::{get_file_status, FileStatus, StatusLoc, StatusType};

pub struct FileComponent {
    event_sender: Sender<ProgramEvent>,
    files: Vec<FileStatus>,
    focused: bool,
    position: usize,
    repo_path: PathBuf,
    state: ListState,
    style: ComponentTheme,
}

// TODO:
//  - Show file diff in window if desired
//  - Files that have some hunks staged while others aren't

impl FileComponent {
    pub fn new(repo_path: PathBuf, event_sender: Sender<ProgramEvent>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            event_sender,
            files: Vec::new(),
            focused: false,
            position: 0,
            repo_path,
            state,
            style: ComponentTheme::default(),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        let list_items: Vec<ListItem> = self
            .files
            .iter()
            .map(|item| {
                let status_type = char::from(item.status_type.clone());
                let style = ComponentTheme::file_status_style(item.status_loc.clone());
                ListItem::new(Span::styled(
                    format!("{} {}", status_type, item.path.clone()),
                    style,
                ))
            })
            .collect();
        let list = TuiList::new(list_items)
            .block(
                Block::default()
                    .title(" Files ")
                    .style(self.style.style())
                    .borders(Borders::ALL)
                    .border_style(self.style.border_style())
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, rect, &mut self.state);
    }

    fn commit(&self) {
        if self.has_files_staged() {
            self.event_sender
                .send(ProgramEvent::Focus(ComponentType::CommitComponent))
                .expect("Send Failed");
        }
    }

    fn has_files_staged(&self) -> bool {
        self.files.iter().any(|file| {
            file.status_type == StatusType::IndexModified
                || file.status_type == StatusType::Added
                || file.status_type == StatusType::Deleted
        })
    }

    fn push(&self) -> Result<(), Error> {
        let remotes = get_remotes(&self.repo_path)?;

        if remotes.is_empty() {
            self.event_sender
                .send(ProgramEvent::Focus(ComponentType::RemotePopupComponent))
                .expect("Send Failed");
            return Ok(());
        }

        let (progress_sender, progress_receiver) = unbounded();
        let repo_path = self.repo_path.clone();
        let event_sender = self.event_sender.clone();

        thread::spawn(move || {
            event_sender
                .send(ProgramEvent::Focus(ComponentType::MessageComponent(
                    "Pushing - 0%".to_string(),
                )))
                .expect("Focus event send failed.");

            // TODO: Don't hardcode remote
            if let Err(err) = push(&repo_path, progress_sender, "origin") {
                event_sender
                    .send(ProgramEvent::Error(err))
                    .expect("Push failure event send failed.");
                return;
            }

            loop {
                let progress = progress_receiver.recv().expect("Receive failed");

                event_sender
                    .send(ProgramEvent::Focus(ComponentType::MessageComponent(
                        format!("Pushing - {}%", progress),
                    )))
                    .expect("Focus event send failed.");

                if progress >= 100 {
                    break;
                }
            }

            // Not sure if getting here will fully indicate success, I think we may need to
            // use the `push_update_reference` callback.
            // For now we will treat getting here as a success unless it hits the fan
            thread::sleep(Duration::from_millis(1000));
            event_sender
                .send(ProgramEvent::Focus(ComponentType::FilesComponent))
                .expect("Focus event send failed.");
        });

        Ok(())
    }

    fn stage_file(&self, all: bool) -> Result<(), Error> {
        if all {
            stage_all(&self.repo_path)?;
        } else if let Some(file) = self.files.get(self.position) {
            stage_file(&self.repo_path, &file.path)?;
        }

        Ok(())
    }

    fn unstage_file(&self, all: bool) -> Result<(), Error> {
        if all {
            unstage_all(&self.repo_path)?;
        } else if let Some(file) = self.files.get(self.position) {
            unstage_file(&self.repo_path, &file.path)?;
        }

        Ok(())
    }
}

impl Component for FileComponent {
    fn update(&mut self) -> Result<(), Error> {
        self.files = get_file_status(&self.repo_path)?;
        if self.files.is_empty() {
            self.files.push(FileStatus {
                path: "Working tree clean".to_string(),
                status_type: StatusType::Unmodified,
                status_loc: StatusLoc::None,
            });
        }
        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
        if !self.focused {
            return Ok(());
        }

        match ev.code {
            KeyCode::Char('j') => self.scroll_down(1),
            KeyCode::Char('k') => self.scroll_up(1),
            KeyCode::Char('a') => self.stage_file(true)?,
            KeyCode::Char('A') => self.unstage_file(true)?,
            KeyCode::Char('s') => self.stage_file(false)?,
            KeyCode::Char('u') => self.unstage_file(false)?,
            KeyCode::Char('c') => self.commit(),
            KeyCode::Char('p') => self.push()?,
            _ => {}
        }

        Ok(())
    }

    fn focus(&mut self, focus: bool) {
        if focus {
            self.style = ComponentTheme::focused();
        } else {
            self.style = ComponentTheme::default();
        }
        self.focused = focus;
    }
}

impl ScrollableComponent for FileComponent {
    fn get_list_length(&self) -> usize {
        self.files.len()
    }
    fn get_position(&self) -> usize {
        self.position
    }
    fn set_position(&mut self, position: usize) {
        self.position = position;
    }
    fn set_state(&mut self, position: usize) {
        self.state.select(Some(position));
    }
}
