use crate::app::ProgramEvent;
use crate::component_style::ComponentTheme;
use crate::components::{Component, ScrollableComponent};
use crate::error::Error;
use crate::git::branch::checkout_local_branch;
use crate::git::commit::revert_commit;
use crate::git::log::{collect_commits, Commit};
use crate::git::repo;

use std::path::PathBuf;

use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState, Paragraph};
use tui::Frame;

use super::ComponentType;

pub struct LogComponent {
    event_sender: Sender<ProgramEvent>,
    filtered_commits: Vec<Commit>,
    focused: bool,
    input: String,
    is_searching: bool,
    commits: Vec<Commit>,
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
            filtered_commits: Vec::new(),
            focused: false,
            input: String::new(),
            is_searching: false,
            commits: Vec::new(),
            position: 0,
            repo_path,
            state,
            style: ComponentTheme::default(),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        let input_constraint = if self.is_searching { 3 } else { 0 };

        let log_block = Block::default()
            .title(" Log ")
            .borders(Borders::ALL)
            .style(self.style.style())
            .border_style(self.style.border_style())
            .border_type(BorderType::Rounded);
        f.render_widget(log_block, rect);

        let container = Layout::default()
            .constraints([Constraint::Length(input_constraint), Constraint::Min(2)].as_ref())
            .direction(Direction::Vertical)
            .margin(1)
            .split(rect);

        let input = Paragraph::new(self.input.as_ref())
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.style.border_style())
                    .border_type(BorderType::Rounded),
            );

        let list_items: Vec<ListItem> = self
            .filtered_commits
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
            .block(Block::default().style(self.style.style()))
            .highlight_style(Style::default().bg(Color::Rgb(48, 48, 48)))
            .highlight_symbol("> ");

        f.render_widget(input, container[0]);
        f.render_stateful_widget(list, container[1], &mut self.state);

        Ok(())
    }

    fn checkout_local_branch(&self) -> Result<(), Error> {
        if let Some(commit) = self.filtered_commits.get(self.position) {
            checkout_local_branch(&self.repo_path, commit.id())?;
        }

        Ok(())
    }

    fn expand_log(&self) {
        if let Some(commit) = self.filtered_commits.get(self.position) {
            self.event_sender
                .send(ProgramEvent::Focus(ComponentType::FullLogComponent(
                    commit.clone(),
                )))
                .expect("Send Failed");
        }
    }

    fn pop_char(&mut self) {
        self.input.pop();
        self.reset_state();

        if self.input.is_empty() {
            self.is_searching = false;
        } else {
            self.filtered_commits = fuzzy_find(&self.commits, &self.input[1..]);
        }
    }

    fn push_char(&mut self, c: char) {
        self.input.push(c);
        self.filtered_commits = fuzzy_find(&self.commits, &self.input[1..]);
        self.reset_state();
    }

    fn revert_commit(&self) -> Result<(), Error> {
        if let Some(commit) = self.filtered_commits.get(self.position) {
            revert_commit(&self.repo_path, commit)?;
        }

        Ok(())
    }
}

impl Component for LogComponent {
    fn update(&mut self) -> Result<(), Error> {
        let repo = repo(&self.repo_path)?;
        let head = repo.head()?.peel_to_commit()?.id();

        self.commits = collect_commits(&self.repo_path, head)?;

        if (self.commits.len() != self.filtered_commits.len()) && !self.is_searching
            || self.input.len() <= 1
        {
            self.filtered_commits = self.commits.clone();
        }
        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
        if !self.focused {
            return Ok(());
        }
        match ev.code {
            // Searching
            KeyCode::Char('j') if ev.modifiers == KeyModifiers::CONTROL => self.scroll_down(1),
            KeyCode::Char('k') if ev.modifiers == KeyModifiers::CONTROL => self.scroll_up(1),
            KeyCode::Char(c) if self.is_searching => self.push_char(c),
            KeyCode::Backspace if self.is_searching => self.pop_char(),
            KeyCode::Esc => {
                self.input.clear();
                self.reset_state();
                self.is_searching = false;
            }

            // Movement
            KeyCode::Char('j') => self.scroll_down(1),
            KeyCode::Char('k') => self.scroll_up(1),
            KeyCode::Char('d') if ev.modifiers == KeyModifiers::CONTROL => self.scroll_down(10),
            KeyCode::Char('u') if ev.modifiers == KeyModifiers::CONTROL => self.scroll_up(10),
            KeyCode::Char('/') => {
                self.is_searching = true;
                self.input.push('/');
            }

            // Program events
            KeyCode::Char('c') => self.checkout_local_branch()?,
            KeyCode::Char('r') => self.revert_commit()?,
            KeyCode::Enter => self.expand_log(),
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

impl ScrollableComponent for LogComponent {
    fn get_list_length(&self) -> usize {
        self.filtered_commits.len()
    }
    fn get_position(&self) -> usize {
        self.position
    }
    fn set_position(&mut self, position: usize) {
        self.position = position;
    }
    fn set_state(&mut self, position: usize) {
        self.state.select(Some(position));
    }
}

fn fuzzy_find(log_list: &[Commit], query: &str) -> Vec<Commit> {
    let matcher = SkimMatcherV2::default();
    log_list
        .iter()
        .filter(|&item| matcher.fuzzy_match(item.message_summary(), query).is_some())
        .cloned()
        .collect::<Vec<_>>()
}
