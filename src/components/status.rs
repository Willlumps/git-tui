use crate::git::gitstatus::DiffStats;

use anyhow::Result;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;

#[allow(unused)]
pub struct StatusComponent<'src> {
    current_branch: String,
    files_changed: usize,
    insertions: usize,
    deletions: usize,
    repo_path: &'src str,
}

impl<'src> StatusComponent<'src> {
    pub fn new(repo_path: &'src str) -> Self {
        Self {
            current_branch: String::new(),
            files_changed: 0,
            insertions: 0,
            deletions: 0,
            repo_path,
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
            Span::styled(format!("  {} ", self.files_changed), Style::default().fg(Color::Blue)),
            Span::styled(format!("  {} ", self.insertions), Style::default().fg(Color::Green)),
            Span::styled(format!("  {} ", self.deletions), Style::default().fg(Color::Red)),
        ]);
        let diff_status = Paragraph::new(text)
            .style(Style::default());
        f.render_widget(diff_status, container[1]);

        let text = Spans::from(vec![
            Span::raw(" On Branch: "),
            Span::styled("Placeholder", Style::default().fg(Color::Yellow)),
        ]);
        let branch_status = Paragraph::new(text)
            .style(Style::default());
        f.render_widget(branch_status, container[0]);
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let stats = DiffStats::get_stats(self.repo_path)?;
        self.files_changed = stats.files_changed;
        self.insertions = stats.insertions;
        self.deletions = stats.deletions;
        Ok(())
    }
}
