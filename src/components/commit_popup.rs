use anyhow::Result;
use crate::git::commit::commit;
use crossterm::event::{KeyCode, KeyEvent};
use super::{centered_rect, Component};
use tui::backend::Backend;
use tui::layout::{Alignment, Rect};
use tui::Frame;
use tui::style::Style;
use tui::widgets::{Clear, Borders, Block, Paragraph};
use std::path::PathBuf;

pub struct CommitPopup {
    input: String,
    visible: bool,
    repo_path: PathBuf,
}

impl CommitPopup {
   pub fn new(repo_path: PathBuf) -> Self {
        Self {
            input: String::new(),
            visible: false,
            repo_path,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        let area = centered_rect(40, 3, rect);
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
        self.visible = false;
        self.input.clear();
    }

    fn commit(&mut self) -> Result<()> {
        if self.input.is_empty() {
            return Ok(());
        }

        commit(&self.repo_path, &self.input)?;
        Ok(())
    }
}

impl Component for CommitPopup {
    fn handle_event(&mut self, ev: KeyEvent) -> Result<()> {
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
