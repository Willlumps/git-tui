use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Text};
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use tui::Frame;

use crate::app::ProgramEvent;
use crate::components::{centered_rect, Component, ComponentType};

// TODO: Expand this to make the errors more reader friendly to better
//       convey what went wrong.
pub struct ErrorComponent {
    event_sender: Sender<ProgramEvent>,
    message: String,
    visible: bool,
}

impl ErrorComponent {
    pub fn new(event_sender: Sender<ProgramEvent>) -> Self {
        Self {
            event_sender,
            message: String::new(),
            visible: false,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        let area = centered_rect(40, 10, rect);

        let block = Block::default()
            .title(Span::styled(" Error ", Style::default().fg(Color::Red)))
            .style(Style::default())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let message_box = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(15),
                    Constraint::Percentage(75),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(area);

        let message = Paragraph::new(Span::raw(&self.message))
            .alignment(tui::layout::Alignment::Center)
            .style(Style::default().fg(Color::White))
            .wrap(tui::widgets::Wrap { trim: true });
        f.render_widget(message, message_box[1]);

        let instructions = Paragraph::new(Text::from("[ESC] - Close Window"))
            .alignment(tui::layout::Alignment::Center)
            .style(Style::default().fg(Color::White));
        f.render_widget(instructions, message_box[2]);

        Ok(())
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    fn reset(&mut self) {
        self.event_sender
            .send(ProgramEvent::Focus(ComponentType::FilesComponent))
            .expect("Focus event send failed.");
        self.visible = false;
        self.message.clear();
    }
}

impl Component for ErrorComponent {
    fn handle_event(&mut self, ev: KeyEvent) -> Result<()> {
        if ev.code == KeyCode::Esc {
            self.reset();
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
