use crossterm::event::{KeyCode, KeyEvent};
use tui::Frame;
use tui::backend::Backend;
use tui::widgets::{Block, Borders, BorderType, List as TuiList, ListItem, ListState, Paragraph};
use tui::style::{Color, Modifier, Style};
use tui::layout::{Alignment, Direction, Layout, Constraint};
use crossterm::{
    event::{poll, read, DisableMouseCapture, Event as CEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};

pub struct LogComponent {
    pub logs: Vec<String>,
    pub state: ListState,
    pub focused: bool,
    pub size: usize,
    pub position: usize,
    pub style: Style,
}

impl LogComponent {
    pub fn new() -> Self {
        let words = vec![
            "ac3aa7e5 Nathan Willer [Date?] Component controls UI updates and events".to_string(),
            "d1a2fe8b Nathan Willer [Date?] Setup main layout view".to_string(),
            "0bde4f13 Nathan Willer [Date?] Organization".to_string(),
            "36154d0d Nathan Willer [Date?] Scrolling lists".to_string(),
        ];

        Self {
            logs: words.clone(),
            state: ListState::default(),
            focused: false,
            size: words.len(),
            position: 0,
            style: Style::default().fg(Color::White),
        }
    }

    pub fn draw<B: tui::backend::Backend>(&mut self, f: &mut tui::Frame<B>, rect: tui::layout::Rect,) -> crossterm::Result<()> {
        let list_items: Vec<ListItem> = self
            .logs
            .iter()
            .map(|item| ListItem::new(item.to_string()))
            .collect();
        let list = TuiList::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.style)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightBlue)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, rect, &mut self.state);

        Ok(())
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
