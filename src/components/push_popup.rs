use super::{centered_rect, Component};
use anyhow::Result;
use crossterm::event::KeyEvent;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Style, Color};
use tui::widgets::{Block, Borders, BorderType, Clear, Paragraph};
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

        let area = centered_rect(30, 3, rect);
        let input = Paragraph::new(self.message.as_ref())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .style(Style::default())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
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
    fn handle_event(&mut self, _ev: KeyEvent) -> Result<()> {
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
