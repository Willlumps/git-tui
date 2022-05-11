use crate::component_style::ComponentTheme;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState, Paragraph};
use tui::Frame;

pub struct StatusComponent {
    branch_name: String,
    files_changed: String,
    insertions: String,
    deletions: String,
}

impl StatusComponent {
    pub fn new() -> Self {
        Self {
            branch_name: String::new(),
            files_changed: String::new(),
            insertions: String::new(),
            deletions: String::new(),
        }
    }

    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
    ) -> crossterm::Result<()> {
        let text = Spans::from(vec![
            Span::styled("  2 ", Style::default().fg(Color::Blue)),
            Span::styled("  22 ", Style::default().fg(Color::Green)),
            Span::styled("  5 ", Style::default().fg(Color::Red)),
        ]);
        let status_container = Paragraph::new(text)
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
                    .border_type(BorderType::Rounded)
                    .title(" Status "),
            );
        f.render_widget(status_container, rect);

        Ok(())
    }
}
