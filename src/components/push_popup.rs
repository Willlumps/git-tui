use crate::components::{centered_rect, Component};
use crate::error::Error;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use tui::Frame;

pub struct PushPopup {
    visible: bool,
    message: String,
}

impl PushPopup {
    pub fn new() -> Self {
        Self {
            visible: false,
            message: String::from("Pushing..."),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        let area = centered_rect(40, 3, rect);
        let input = Paragraph::new(self.message.as_ref())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .style(Style::default())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(tui::layout::Alignment::Center);

        f.render_widget(Clear, area);
        f.render_widget(input, area);
        Ok(())
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
    }
}

impl Component for PushPopup {
    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
        match ev.code {
            KeyCode::Char('q') if ev.modifiers == KeyModifiers::CONTROL => {
                self.focus(false);
                return Ok(());
            }
            _ => {}
        }
        Ok(())
    }

    fn focus(&mut self, focus: bool) {
        self.message = String::from("Pushing...");
        self.visible = focus;
    }

    fn update(&mut self) -> Result<()> {
        Ok(())
    }
}
