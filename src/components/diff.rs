use crate::git::gitdiff::get_diff;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use git2::DiffLine as Git2DiffLine;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState, Paragraph};
use tui::Frame;

pub struct DiffComponent {
    pub diffs: Vec<DiffLine>,
    pub state: ListState,
    pub focused: bool,
    pub position: usize,
    pub style: Style,
    pub path: String,
    size: usize,
    window_min: usize,
    window_max: usize,
    height: usize,
    first_render: bool,
}

pub struct DiffLine {
    pub content: String,
    pub origin: char,
    pub style: Style,
}

impl DiffLine {
    pub fn origin(&self) -> char {
        self.origin
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn content(&self) -> &String {
        &self.content
    }
}

impl DiffComponent {
    pub fn new(repo_path: &str) -> Self {
        let diffs = get_diff(repo_path).unwrap();
        let len = diffs.len();

        Self {
            diffs,
            state: ListState::default(),
            focused: false,
            position: 0,
            style: Style::default().fg(Color::White),
            path: repo_path.to_string(),
            size: len,
            window_min: 0,
            window_max: 0,
            height: 0,
            first_render: true,
        }
    }
}

impl DiffComponent {
    pub fn draw<B: tui::backend::Backend>(
        &mut self,
        f: &mut tui::Frame<B>,
        rect: tui::layout::Rect,
    ) -> crossterm::Result<()> {
        self.update_diff();
        self.height = (f.size().height as usize) - 4;
        if self.first_render {
            self.render_diff();
        }

        let list_items: Vec<ListItem> = self
            .diffs
            .iter()
            .map(|item| {
                let content = match item.origin() {
                    '-' => format!("-{}", item.content()),
                    '+' => format!("+{}", item.content()),
                    _ => item.content().to_string(),
                };
                let text = Span::styled(content, item.style());
                ListItem::new(text)
            })
            .collect();
        let list = TuiList::new(list_items)
            .block(
                Block::default()
                    .title(" Diff ")
                    .borders(Borders::ALL)
                    .border_style(self.style)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
            );

        f.render_stateful_widget(list, rect, &mut self.state);

        Ok(())
    }

    pub fn handle_event(&mut self, ev: KeyEvent) {
        if !self.focused {
            return;
        }
        match ev.code {
            KeyCode::Char('j') => {
                self.decrement_position(1);
            },
            KeyCode::Char('k') => {
                self.increment_position(1);
            },
            KeyCode::Char('d') if ev.modifiers == KeyModifiers::CONTROL => {
                self.decrement_position(self.height / 2);
            },
            KeyCode::Char('u') if ev.modifiers == KeyModifiers::CONTROL => {
                self.increment_position(self.height / 2);
            },
            _ => {}
        }
    }

    pub fn update_diff(&mut self) {
        let path = &self.path;
        let diff = get_diff(path.as_ref()).unwrap();
        if diff.len() != self.diffs.len() {
            self.render_diff();
        }
        self.diffs = diff;
        self.size = self.diffs.len();
    }

    pub fn focus(&mut self, focus: bool) {
        if focus {
            self.style = Style::default().fg(Color::Yellow);
        } else {
            self.style = Style::default().fg(Color::White);
        }
        self.focused = focus;
    }

    fn render_diff(&mut self) {
        self.first_render = false;
        self.window_min = 0;
        self.window_max = self.height - 1;
        self.state.select(Some(self.window_max));
    }

    fn increment_position(&mut self, i: usize) {
        self.position = self.window_min;
        self.window_min = self.window_min.saturating_sub(i);
        self.position = self.position.saturating_sub(i);

        if self.position != 0 {
            self.window_max -= i;
            if self.window_max < self.height - 4 {
                self.window_max = self.height - 4;
            }
        }
        self.state.select(Some(self.position));
    }

    fn decrement_position(&mut self, i: usize) {
        self.position = self.window_max;
        if self.position < self.size - 1 {
            self.position += i;
            self.window_max += i;
            self.window_min += i;

            if self.position > self.size - 1 {
                self.position = self.size - 1;
                self.window_max = self.size - 1;
                self.window_min = self.size - self.height;

            }
        }
        self.state.select(Some(self.position));
    }
}
