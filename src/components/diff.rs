use crate::component_style::ComponentTheme;
use crate::git::git_diff::get_diff;
use crate::list_window::{ListWindow, ScrollDirection};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::Style;
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState};
use tui::Frame;

use std::path::PathBuf;

use super::Component;

pub struct DiffComponent {
    pub diffs: Vec<DiffLine>,
    pub state: ListState,
    pub focused: bool,
    style: ComponentTheme,
    repo_path: PathBuf,
    window: ListWindow,
    first_update: bool,
}

#[derive(Debug, PartialEq)]
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
    pub fn new(repo_path: PathBuf) -> Self {
        let diffs = get_diff(&repo_path).unwrap();
        let len = diffs.len();

        Self {
            diffs,
            state: ListState::default(),
            focused: false,
            style: ComponentTheme::default(),
            repo_path,
            window: ListWindow::new(0, 0, 0, len, 0),
            first_update: true,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        self.window.set_height((f.size().height as usize) - 4);

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
        let list = TuiList::new(list_items).block(
            Block::default()
                .title(" Diff ")
                .style(self.style.style())
                .borders(Borders::ALL)
                .border_style(self.style.border_style())
                .border_type(BorderType::Rounded),
        );

        f.render_stateful_widget(list, rect, &mut self.state);

        Ok(())
    }

    fn render_diff(&mut self) {
        self.window.reset();
        self.state.select(self.window.position());
    }

    fn scroll_up(&mut self, i: usize) {
        self.window.scroll(ScrollDirection::Up, i);
        self.state.select(self.window.position());
    }

    fn scroll_down(&mut self, i: usize) {
        self.window.scroll(ScrollDirection::Down, i);
        self.state.select(self.window.position());
    }
}

impl Component for DiffComponent {
    fn update(&mut self) -> Result<()> {
        if self.first_update && self.window.height() > 0 {
            self.first_update = false;
            self.window.reset();
        }

        let path = &self.repo_path;
        let diff = get_diff(path)?;
        if diff.len() != self.diffs.len() {
            self.render_diff();
            self.diffs = diff;
            self.window.set_size(self.diffs.len());
        }
        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) -> Result<()> {
        if !self.focused {
            return Ok(());
        }
        match ev.code {
            KeyCode::Char('j') => {
                self.scroll_down(1);
            },
            KeyCode::Char('k') => {
                self.scroll_up(1);
            },
            KeyCode::Char('d') if ev.modifiers == KeyModifiers::CONTROL => {
                let height = self.window.height();
                self.scroll_down(height / 2);
            },
            KeyCode::Char('u') if ev.modifiers == KeyModifiers::CONTROL => {
                let height = self.window.height();
                self.scroll_up(height / 2);
            },
            _ => {}
        }
        Ok(())
    }

    fn focus(&mut self, focus: bool) {
        if focus {
            self.style = ComponentTheme::focused();
        } else {
            self.style = ComponentTheme::default();
        }
        self.focused = focus;
    }
}
