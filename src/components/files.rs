use crate::component_style::ComponentTheme;
use crate::git::git_status::get_file_status;
use crate::git::git_status::FileStatus;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState};
use tui::Frame;

use super::Component;
use std::path::PathBuf;

pub struct FileComponent {
    pub files: Vec<FileStatus>,
    pub state: ListState,
    pub focused: bool,
    pub size: usize,
    pub position: usize,
    pub style: ComponentTheme,
    repo_path: PathBuf,
}

// TODO:
//  - Add and Restore files for committing
//  - Show file diff in window if desired

impl FileComponent {
    pub fn new(repo_path: PathBuf) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            files: Vec::new(),
            state,
            focused: false,
            size: 0,
            position: 0,
            style: ComponentTheme::default(),
            repo_path,
        }
    }

    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
    ) -> Result<()> {
        let list_items: Vec<ListItem> = self
            .files
            .iter()
            .map(|item| {
                let status_type = char::from(item.status_type.clone());
                let style = ComponentTheme::file_status_style(item.status_loc.clone());
                ListItem::new(Span::styled(format!("{} {}", status_type, item.path.clone()), style))
            })
            .collect();
        let list = TuiList::new(list_items)
            .block(
                Block::default()
                    .title(" Files ")
                    .style(self.style.style())
                    .borders(Borders::ALL)
                    .border_style(self.style.border_style())
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, rect, &mut self.state);

        Ok(())
    }

    fn increment_position(&mut self) {
        if self.position != 0 {
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
}

impl Component for FileComponent {
    fn update(&mut self) -> Result<()> {
        self.files = get_file_status(&self.repo_path)?;
        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) {
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

    fn focus(&mut self, focus: bool) {
        if focus {
            self.style = ComponentTheme::focused();
        } else {
            self.style = ComponentTheme::default();
        }
        self.focused = focus;
    }
}
