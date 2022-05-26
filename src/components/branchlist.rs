use std::path::PathBuf;
use std::sync::mpsc::Sender;

use crate::app::ProgramEvent;
use crate::component_style::ComponentTheme;
use crate::error::Error;
use crate::git::git_branch::{checkout_branch, get_branches, Branch};
use crate::ComponentType;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState, Paragraph};
use tui::Frame;

use super::Component;

pub struct BranchComponent {
    branches: Vec<Branch>,
    filtered_branches: Vec<Branch>,
    state: ListState,
    focused: bool,
    position: usize,
    style: ComponentTheme,
    input: String,
    repo_path: PathBuf,
    event_sender: Sender<ProgramEvent>,
}

impl BranchComponent {
    pub fn new(repo_path: PathBuf, event_sender: Sender<ProgramEvent>) -> BranchComponent {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            branches: Vec::new(),
            filtered_branches: Vec::new(),
            state,
            focused: false,
            position: 0,
            style: ComponentTheme::default(),
            input: String::new(),
            repo_path,
            event_sender,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        let branch_block = Block::default()
            .title(" Branches ")
            .style(self.style.style())
            .borders(Borders::ALL)
            .border_style(self.style.border_style())
            .border_type(BorderType::Rounded);
        f.render_widget(branch_block, rect);

        let branch_container = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
            .split(rect);

        let input = Paragraph::new(self.input.as_ref())
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Search ")
                    .title_alignment(Alignment::Center),
            );

        let list_items: Vec<ListItem> = self
            .branches
            .iter()
            .map(|item| ListItem::new(&*item.name))
            .collect();
        let list = TuiList::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_widget(input, branch_container[0]);
        f.render_stateful_widget(list, branch_container[1], &mut self.state);

        Ok(())
    }

    fn increment_position(&mut self) {
        self.position = self.position.saturating_sub(1);
        self.state.select(Some(self.position));
    }

    fn decrement_position(&mut self) {
        if self.position < self.branches.len() - 1 {
            self.position += 1;
            self.state.select(Some(self.position));
        }
    }

    // fn reset_state(&mut self) {
    //     self.position = 0;
    //     self.state.select(Some(0));
    // }
}

impl Component for BranchComponent {
    fn update(&mut self) -> Result<()> {
        self.branches = get_branches(&self.repo_path)?;
        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
        if !self.focused {
            return Ok(());
        }

        match ev.code {
            KeyCode::Char('j') if ev.modifiers == KeyModifiers::CONTROL => {
                self.decrement_position();
            }
            KeyCode::Char('k') if ev.modifiers == KeyModifiers::CONTROL => {
                self.increment_position();
            }
            KeyCode::Char('c') => {
                if let Some(branch) = self.branches.get(self.position) {
                    checkout_branch(&self.repo_path, &branch.name)?;
                }
            }
            KeyCode::Char('n') => {
                self.event_sender
                    .send(ProgramEvent::Focus(ComponentType::BranchPopupComponent))
                    .expect("Send failed.");
            }
            // KeyCode::Char(c) => {
            //     self.input.push(c);
            //     self.filtered_branches = fuzzy_find(&self.branches, &self.input);
            //     self.reset_state();
            // }
            // KeyCode::Backspace => {
            //     self.input.pop();
            //     self.filtered_branches = fuzzy_find(&self.branches, &self.input);
            //     self.reset_state();
            // }
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

#[allow(dead_code)]
fn fuzzy_find(filtered_list: &[String], query: &str) -> Vec<String> {
    let matcher = SkimMatcherV2::default();
    filtered_list
        .iter()
        .filter(|&item| matcher.fuzzy_match(item, query).is_some())
        .cloned()
        .collect::<Vec<_>>()
}
