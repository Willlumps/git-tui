use crate::app::{GitEvent, ProgramEvent};
use crate::components::{centered_rect, Component, ComponentType};
use crate::error::Error;
use crate::git::commit::commit;

use std::path::PathBuf;

use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
    cursor_visible: bool,
    cursor_position: (u16, u16),
}

impl CommitPopup {
    pub fn new(repo_path: PathBuf, event_sender: Sender<ProgramEvent>) -> Self {
        Self {
            input: String::new(),
            visible: false,
            event_sender,
            repo_path,
            cursor_visible: false,
            cursor_position: (0, 0),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        let area = centered_rect(100, 3, rect);

        if !self.cursor_visible {
            f.set_cursor(area.x + 1, area.y + 1);
            self.cursor_position = (area.x + 1, area.y + 1);
            self.cursor_visible = true;
        } else {
            f.set_cursor(self.cursor_position.0, self.cursor_position.1);
        }

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
        self.cursor_visible = false;
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
            KeyCode::Char('w') if ev.modifiers == KeyModifiers::CONTROL => {
                let input_ref = self.input.as_str().trim();
                let last_space_index = input_ref.rfind(' ');
                match last_space_index {
                    Some(index) => {
                        let cursor_index = self.cursor_position.0 - (self.input.len() - index) as u16;
                        self.input = self.input[0..index].to_string();
                        self.cursor_position.0 = cursor_index;
                    }
                    None => {
                        self.cursor_position.0 -= self.input.len() as u16;
                        self.input.clear();
                    }
                }
            }
            KeyCode::Char(c) => {
                if self.input.len() < 95 {
                    self.cursor_position.0 += 1;
                    self.input.push(c);
                }
            }
            KeyCode::Backspace => {
                if !self.input.is_empty() {
                    self.cursor_position.0 -= 1;
                    self.input.pop();
                }
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
