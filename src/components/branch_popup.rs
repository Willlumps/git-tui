use super::{centered_rect, Component, ComponentType};
use crate::app::{GitEvent, ProgramEvent};
use crate::error::Error;
use crate::git::git_branch::{create_branch, checkout_branch};
use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;
use tui::backend::Backend;
use tui::layout::{Alignment, Rect};
use tui::style::Style;
use tui::widgets::{Block, Borders, Clear, Paragraph};
use tui::Frame;

pub struct BranchPopup {
    input: String,
    visible: bool,
    event_sender: Sender<ProgramEvent>,
    repo_path: PathBuf,
}

impl BranchPopup {
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
                    .title(" Create Branch ")
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
            .send(ProgramEvent::Focus(ComponentType::BranchComponent))
            .expect("Focus event send failed.");
        self.visible = false;
        self.input.clear();
    }

    #[allow(clippy::single_char_pattern)]
    fn create_branch(&mut self, input: &str) -> Result<(), Error> {
        if input.is_empty() {
            return Ok(());
        }
        create_branch(&self.repo_path, input)?;
        checkout_branch(&self.repo_path, input)?;
        Ok(())
    }
}

impl Component for BranchPopup {
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
                let input = self.input.clone();
                self.reset();
                self.create_branch(&input)?;
                self.event_sender
                    .send(ProgramEvent::Git(GitEvent::RefreshBranchList))
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

    fn update(&mut self) -> Result<()> {
        Ok(())
    }
}
