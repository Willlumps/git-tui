use crate::app::ProgramEvent;
use crate::component_style::ComponentTheme;
use crate::components::Component;
use crate::error::Error;
use crate::git::branch::checkout_local_branch;
use crate::git::log::{fetch_history, Commit};

use std::path::PathBuf;

use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState};
use tui::Frame;

use super::ComponentType;

pub struct LogComponent {
    event_sender: Sender<ProgramEvent>,
    focused: bool,
    logs: Vec<Commit>,
    position: usize,
    repo_path: PathBuf,
    state: ListState,
    style: ComponentTheme,
}

impl LogComponent {
    pub fn new(repo_path: PathBuf, event_sender: Sender<ProgramEvent>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            event_sender,
            focused: false,
            logs: Vec::new(),
            position: 0,
            repo_path,
            state,
            style: ComponentTheme::default(),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        let list_items: Vec<ListItem> = self
            .logs
            .iter()
            .map(|item| {
                let text = Spans::from(vec![
                    Span::styled(item.shorthand_id(), Style::default().fg(Color::Green)),
                    Span::raw(" "),
                    Span::raw(item.message_summary()),
                ]);
                ListItem::new(text)
            })
            .collect();
        let list = TuiList::new(list_items)
            .block(
                Block::default()
                    .title(" Log ")
                    .borders(Borders::ALL)
                    .style(self.style.style())
                    .border_style(self.style.border_style())
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, rect, &mut self.state);

        Ok(())
    }

    fn scroll_up(&mut self, amount: usize) {
        self.position = self.position.saturating_sub(amount);
        self.state.select(Some(self.position));
    }

    fn scroll_down(&mut self, amount: usize) {
        if self.position < self.logs.len() - amount - 1 {
            self.position += amount;
        } else {
            self.position = self.logs.len() - 1;
        }
        self.state.select(Some(self.position));
    }
}

impl Component for LogComponent {
    fn update(&mut self) -> Result<(), Error> {
        self.logs = fetch_history(&self.repo_path)?;
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
            KeyCode::Char('c') => {
                if let Some(commit) = self.logs.get(self.position) {
                    checkout_local_branch(&self.repo_path, commit.id())?;
                }
            }
            KeyCode::Char('d') if ev.modifiers == KeyModifiers::CONTROL => {
                self.scroll_down(10);
            }
            KeyCode::Char('u') if ev.modifiers == KeyModifiers::CONTROL => {
                self.scroll_up(10);
            }
            KeyCode::Enter => {
                if let Some(commit) = self.logs.get(self.position) {
                    self.event_sender
                        .send(ProgramEvent::Focus(ComponentType::FullLogComponent(
                            commit.clone(),
                        )))
                        .expect("Send Failed");
                }
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
