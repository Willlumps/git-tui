use crate::component_style::ComponentTheme;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState, Paragraph};
use tui::Frame;

use std::error::Error;

pub struct FileComponent {
    pub files: Vec<String>, // TODO
    pub state: ListState,
    pub focused: bool,
    pub size: usize,
    pub position: usize,
    pub style: ComponentTheme,
}

impl FileComponent {
    pub fn new() -> Self {
        Self {
            files: vec!["Placeholder1".to_string(), "Placeholder2".to_string()], // TODO
            state: ListState::default(),
            focused: false,
            size: 0,
            position: 0,
            style: ComponentTheme::default(),
        }
    }

    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        let list_items: Vec<ListItem> = self
            .files
            .iter()
            .map(|item| ListItem::new(item.to_string()))
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
        if focus {
            self.style = ComponentTheme::focused();
        } else {
            self.style = ComponentTheme::default();
        }
        self.focused = focus;
    }
}
