use crate::app::ProgramEvent;
use crate::components::ComponentType;
use crate::error::Error;

use super::{centered_rect, Component};
use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent};
use git2::{ErrorClass, ErrorCode};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Text};
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use tui::Frame;

// TODO: Expand this to make the errors more reader friendly to better
//       convey what went wrong.
pub struct ErrorComponent {
    code: ErrorCode,
    class: ErrorClass,
    message: String,
    event_sender: Sender<ProgramEvent>,
    visible: bool,
}

impl ErrorComponent {
    pub fn new(event_sender: Sender<ProgramEvent>) -> Self {
        Self {
            code: ErrorCode::GenericError,
            class: ErrorClass::None,
            message: String::new(),
            event_sender,
            visible: false,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }
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

        let code = Paragraph::new(Span::raw(format!(
            "{:?} Error ({:?})",
            &self.class, &self.code
        )))
        .alignment(tui::layout::Alignment::Center)
        .style(Style::default().fg(Color::White))
        .wrap(tui::widgets::Wrap { trim: true });
        f.render_widget(code, message_box[0]);

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

    pub fn set_git_error(&mut self, error: git2::Error) {
        self.code = error.code();
        self.class = error.class();
        self.message = error.message().to_string();
    }

    fn reset(&mut self) {
        self.event_sender
            .send(ProgramEvent::Focus(ComponentType::FilesComponent))
            .expect("Focus event send failed.");
        self.visible = false;
        self.message.clear();
        self.code = ErrorCode::GenericError;
        self.class = ErrorClass::None;
    }
}

impl Component for ErrorComponent {
    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
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
