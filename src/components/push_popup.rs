use std::sync::mpsc::Sender;

use super::{centered_rect, Component, ComponentType};
use anyhow::Result;
use crossterm::event::KeyEvent;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::{Block, Borders, Clear, Paragraph};
use tui::Frame;

pub struct PushPopup {
    visible: bool,
    _event_sender: Sender<ComponentType>,
}

impl PushPopup {
    pub fn new(event_sender: Sender<ComponentType>) -> Self {
        Self {
            visible: false,
            _event_sender: event_sender,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        let area = centered_rect(10, 3, rect);
        let input = Paragraph::new("Pushing...")
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
            );

        f.render_widget(Clear, area);
        f.render_widget(input, area);
        Ok(())
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}

impl Component for PushPopup {
    fn handle_event(&mut self, _ev: KeyEvent) -> Result<()> {
        Ok(())
    }

    fn focus(&mut self, focus: bool) {
        self.visible = focus;
    }

    fn update(&mut self) -> Result<()> {
        Ok(())
    }
}
