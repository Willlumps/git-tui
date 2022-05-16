use crate::git::git_diff::{DiffWindow, get_diff_stats};

use anyhow::Result;
use crossterm::event::KeyEvent;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;

use std::path::PathBuf;

use super::Component;

#[allow(unused)]
pub struct StatusComponent {
    repo_path: PathBuf,
    status: DiffWindow,
}

impl StatusComponent {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            status: DiffWindow::default(),
        }
    }

    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
    ) -> Result<()> {
        let status_block = Block::default()
            .title(" Status ")
            .style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded);
        f.render_widget(status_block, rect);

        let container = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(rect);

        let text = Spans::from(vec![
            Span::styled(format!("  {} ", self.status.files_changed), Style::default().fg(Color::Blue)),
            Span::styled(format!("  {} ", self.status.insertions), Style::default().fg(Color::Green)),
            Span::styled(format!("  {} ", self.status.deletions), Style::default().fg(Color::Red)),
        ]);
        let diff_status = Paragraph::new(text)
            .style(Style::default());
        f.render_widget(diff_status, container[1]);

        let text = Spans::from(vec![
            Span::raw(" On Branch: "),
            Span::styled(&self.status.branch, Style::default().fg(Color::Yellow)),
        ]);
        let branch_status = Paragraph::new(text)
            .style(Style::default());
        f.render_widget(branch_status, container[0]);
        Ok(())
    }
}

impl Component for StatusComponent {
    fn update(&mut self) -> Result<()> {
        self.status = get_diff_stats(&self.repo_path)?;
        Ok(())
    }

    // no-op
    fn handle_event(&mut self, _ev: KeyEvent) {}
    fn focus(&mut self, _focus: bool) {}
}
