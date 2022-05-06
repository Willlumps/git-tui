use crate::git::gitlog::{Commit, GitLog};

use crossterm::event::{KeyCode, KeyEvent};
use tui::Frame;
use tui::backend::Backend;
use tui::widgets::{Block, Borders, BorderType, List as TuiList, ListItem, ListState, Paragraph};
use tui::style::{Color, Modifier, Style};
use tui::layout::{Alignment, Direction, Layout, Constraint};
use tui::text::{Span, Spans};
use crossterm::{
    event::{poll, read, DisableMouseCapture, Event as CEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};

pub struct LogComponent {
    pub logs: Vec<Commit>,
    pub state: ListState,
    pub focused: bool,
    pub size: usize,
    pub position: usize,
    pub style: Style,
}

impl LogComponent {
    pub fn new() -> Self {
        let mut git_log = GitLog::new("/Users/reina/school/groupwork/capstone".to_string());
        git_log.get_history();

        Self {
            logs: git_log.history.clone(),
            state: ListState::default(),
            focused: false,
            size: git_log.history.len(),
            position: 0,
            style: Style::default().fg(Color::White),
        }
    }

    pub fn draw<B: tui::backend::Backend>(&mut self, f: &mut tui::Frame<B>, rect: tui::layout::Rect,) -> crossterm::Result<()> {
        let list_items: Vec<ListItem> = self
            .logs
            .iter()
            .map(|item| {
                let text = vec![
                    Spans::from(vec![
                        Span::styled(item.get_id(), Style::default().fg(Color::Green)),
                        Span::raw(" ["),
                        Span::raw(item.get_author()),
                        Span::raw("] "),
                        Span::raw(item.get_message()),
                    ]),
                ];
                ListItem::new(text)
            })
            .collect();
        let list = TuiList::new(list_items)
            .block(
                Block::default()
                    .title(" Logs ")
                    .borders(Borders::ALL)
                    .border_style(self.style)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, rect, &mut self.state);

        Ok(())
    }

    pub fn handle_event(&mut self, ev: KeyEvent) {
        if !self.focused {
            return;
        }
        match ev.code {
            KeyCode::Char('j') if ev.modifiers == KeyModifiers::CONTROL => {
                self.decrement_position();
            }
            KeyCode::Char('k') if ev.modifiers == KeyModifiers::CONTROL => {
                self.increment_position();
            }
            _ => {}
        }
    }

    fn get_position(&self) -> usize {
        self.position
    }

    fn increment_position(&mut self) {
        if self.get_position() != 0 {
            self.position -= 1;
            self.state.select(Some(self.position));
        }
    }

    fn decrement_position(&mut self) {
        if self.position < self.size - 1 {
            self.position += 1;
            self.state.select(Some(self.position));
        }
    }

    pub fn focus(&mut self, focus: bool) {
        // TODO: ?
        if focus {
            self.style = Style::default().fg(Color::Yellow);
        } else {
            self.style = Style::default().fg(Color::White);
        }
        self.focused = focus;
    }
}
