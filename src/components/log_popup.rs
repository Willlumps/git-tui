use crate::components::{centered_rect, Component, ComponentType};
use crate::ProgramEvent;
use crate::error::Error;
use crate::git::log::Commit;

use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Text;
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use tui::Frame;

pub struct LogPopup {
    visible: bool,
    commit: Commit,
    event_sender: Sender<ProgramEvent>,
}

impl LogPopup {
    pub fn new(event_sender: Sender<ProgramEvent>) -> Self {
        Self {
            visible: false,
            commit: Commit::new(),
            event_sender,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        let area = centered_rect(80, 7, rect);

        let mut log = Text::styled(
            format!(" commit: {}", self.commit.id()),
            Style::default().fg(Color::Yellow),
        );
        log.extend(Text::raw(format!(
            " Author: {} <{}>",
            self.commit.author(),
            self.commit.email()
        )));
        log.extend(Text::raw(format!(" Date:   {}", self.commit.time())));
        log.extend(Text::raw(format!("\n     {}\n", self.commit.message())));

        let input = Paragraph::new(log)
            .style(Style::default().fg(Color::White))
            .wrap(tui::widgets::Wrap { trim: false })
            .block(
                Block::default()
                    .style(Style::default())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(tui::layout::Alignment::Left);

        f.render_widget(Clear, area);
        f.render_widget(input, area);
        Ok(())
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_commit(&mut self, commit: Commit) {
        self.commit = commit;
    }

    fn reset(&mut self) {
        self.event_sender
            .send(ProgramEvent::Focus(ComponentType::LogComponent))
            .expect("Focus event send failed.");
        self.visible = false;
    }
}

impl Component for LogPopup {
    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
        if ev.code == KeyCode::Esc {
            self.reset()
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
