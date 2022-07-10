use crate::component_style::ComponentTheme;
use crate::components::Component;
use crate::error::Error;
use crate::git::diff::{DiffLine, get_staged};
use crate::list_window::{ListWindow, ScrollDirection};

use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState};
use tui::Frame;

pub struct DiffStagedComponent {
    diffs: Vec<DiffLine>,
    first_update: bool,
    focused: bool,
    repo_path: PathBuf,
    state: ListState,
    style: ComponentTheme,
    window: ListWindow,
}

impl DiffStagedComponent {
    pub fn new(repo_path: PathBuf) -> Self {
        let diffs = get_staged(&repo_path).unwrap();
        let len = diffs.len();

        Self {
            diffs,
            first_update: true,
            focused: false,
            repo_path,
            state: ListState::default(),
            style: ComponentTheme::default(),
            window: ListWindow::new(0, 0, 0, len, 0),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        self.window.set_height(((f.size().height as usize) - 4) / 2);

        let list_items: Vec<ListItem> = self
            .diffs
            .iter()
            .map(|item| {
                let content = match item.origin() {
                    '-' => format!("-{}", item.content()),
                    '+' => format!("+{}", item.content()),
                    _ => format!(" {}", item.content()),
                };
                let text = Span::styled(content, item.style());
                ListItem::new(text)
            })
            .collect();

        let list = TuiList::new(list_items).block(
            Block::default()
                .title(" Staged ")
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

impl Component for DiffStagedComponent {
    fn update(&mut self) -> Result<(), Error> {
        if self.first_update && self.window.height() > 0 {
            self.first_update = false;
            self.window.reset();
        }

        let path = &self.repo_path;
        let diff = get_staged(path)?;
        if diff.len() != self.diffs.len() {
            self.render_diff();
            self.diffs = diff;
            self.window.set_size(self.diffs.len());
        }
        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
        if !self.focused {
            return Ok(());
        }
        match ev.code {
            KeyCode::Char('j') => {
                self.scroll_down(1);
            }
            KeyCode::Char('k') => {
                self.scroll_up(1);
            }
            KeyCode::Char('d') if ev.modifiers == KeyModifiers::CONTROL => {
                let height = self.window.height();
                self.scroll_down(height / 2);
            }
            KeyCode::Char('u') if ev.modifiers == KeyModifiers::CONTROL => {
                let height = self.window.height();
                self.scroll_up(height / 2);
            }
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
