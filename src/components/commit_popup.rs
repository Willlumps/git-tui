use crate::app::{GitEvent, ProgramEvent};
use crate::components::{centered_rect, Component, ComponentType};
use crate::error::Error;
use crate::git::commit::commit;

use std::path::PathBuf;

use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent};
use tui::backend::Backend;
use tui::layout::{Alignment, Rect};
use tui::style::Style;
use tui::widgets::{Block, Borders, Clear, Paragraph};
use tui::Frame;

pub struct CommitPopup {
    input: String,
    visible: bool,
    event_sender: Sender<ProgramEvent>,
    repo_path: PathBuf,
}

impl CommitPopup {
    pub fn new(repo_path: PathBuf, event_sender: Sender<ProgramEvent>) -> Self {
        Self {
            input: String::new(),
            visible: false,
            event_sender,
            repo_path,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        let area = centered_rect(80, 3, rect);
        let input = Paragraph::new(self.input.as_ref())
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Commit ")
                    .title_alignment(Alignment::Left),
            );

        f.render_widget(Clear, area);
        f.render_widget(input, area);
        Ok(())
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    fn reset(&mut self) {
        self.event_sender
            .send(ProgramEvent::Focus(ComponentType::FilesComponent))
            .expect("Focus event send failed.");
        self.visible = false;
        self.input.clear();
    }

    fn commit(&mut self) -> Result<(), Error> {
        if self.input.is_empty() {
            return Ok(());
        }

        commit(&self.repo_path, &self.input)?;
        Ok(())
    }
}

impl Component for CommitPopup {
    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
        if !self.visible {
            return Ok(());
        }

        match ev.code {
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                self.commit()?;
                self.reset();
                self.event_sender
                    .send(ProgramEvent::Git(GitEvent::RefreshCommitLog))
                    .expect("Send failed");
            }
            KeyCode::Esc => {
                self.reset();
            }
            _ => {}
        }

        Ok(())
    }

    fn focus(&mut self, focus: bool) {
        self.visible = focus;
    }

    fn update(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
