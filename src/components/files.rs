use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use anyhow::Result;
use crossbeam::channel::Sender;
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
use crate::git::remote::{get_remote, push};
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
    input_lock: Arc<RwLock<AtomicBool>>,
}

// TODO:
//  - Show file diff in window if desired
//  - Files that have some hunks staged while others aren't

impl FileComponent {
    pub fn new(
        repo_path: PathBuf,
        event_sender: Sender<ProgramEvent>,
        input_lock: Arc<RwLock<AtomicBool>>,
    ) -> Self {
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
            input_lock,
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

    fn commit_full(&self) -> Result<()> {
        {
            let handle = self.input_lock.write().unwrap();
            handle.store(true, Ordering::Relaxed);
        }

        let mut commit_msg = std::process::Command::new("git")
            .arg("commit")
            .spawn()?;

        commit_msg.wait()?;
        // hackity hack hack
        self.event_sender.send(ProgramEvent::ClearTerminal).expect("Send failed");

        {
            let handle = self.input_lock.write().unwrap();
            handle.store(false, std::sync::atomic::Ordering::Relaxed);
        }

        Ok(())
    }

    fn has_files_staged(&self) -> bool {
        self.files.iter().any(|file| {
            file.status_type == StatusType::IndexModified
                || file.status_type == StatusType::Added
                || file.status_type == StatusType::Deleted
        })
    }

    fn push(&self) -> Result<()> {
        match get_remote(&self.repo_path)? {
            Some(remote_name) => {
                push(
                    self.event_sender.clone(),
                    self.repo_path.clone(),
                    remote_name,
                )?;
            }
            None => {
                self.event_sender
                    .send(ProgramEvent::Focus(ComponentType::RemotePopupComponent))
                    .expect("Send Failed");
            }
        }

        Ok(())
    }

    fn stage_file(&self, all: bool) -> Result<()> {
        if all {
            stage_all(&self.repo_path)?;
        } else if let Some(file) = self.files.get(self.position) {
            stage_file(&self.repo_path, &file.path)?;
        }

        Ok(())
    }

    fn unstage_file(&self, all: bool) -> Result<()> {
        if all {
            unstage_all(&self.repo_path)?;
        } else if let Some(file) = self.files.get(self.position) {
            unstage_file(&self.repo_path, &file.path)?;
        }

        Ok(())
    }
}

impl Component for FileComponent {
    fn update(&mut self) -> Result<()> {
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

    fn handle_event(&mut self, ev: KeyEvent) -> Result<()> {
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
            KeyCode::Char('C') => self.commit_full()?,
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
