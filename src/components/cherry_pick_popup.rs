use std::path::PathBuf;

use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Clear, List as TuiList, ListItem, ListState};
use tui::Frame;

use crate::components::{centered_rect, Component, ComponentType, ScrollableComponent};
use crate::error::Error;
use crate::git::commit::cherry_pick;
use crate::git::log::Commit;
use crate::git::repo;
use crate::ProgramEvent;

pub struct CherryPickPopup {
    commits: Vec<Commit>,
    event_sender: Sender<ProgramEvent>,
    position: usize,
    repo_path: PathBuf,
    selected_commits: Vec<String>,
    state: ListState,
    visible: bool,
}
impl CherryPickPopup {
    pub fn new(repo_path: PathBuf, event_sender: Sender<ProgramEvent>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            commits: Vec::new(),
            event_sender,
            position: 0,
            repo_path,
            selected_commits: Vec::new(),
            state,
            visible: false,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        let area = centered_rect(80, 40, rect);

        let list_items: Vec<ListItem> = self
            .commits
            .iter()
            .map(|item| {
                let (selected, style) = if self.selected_commits.contains(item.id()) {
                    ("[x] ".to_string(), Style::default().fg(Color::Yellow))
                } else {
                    ("[ ] ".to_string(), Style::default())
                };

                let text = Spans::from(vec![
                    Span::styled(selected, style),
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
                    .title(" Cherry-Pick ")
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().bg(Color::Rgb(48, 48, 48)));

        f.render_widget(Clear, area);
        f.render_stateful_widget(list, area, &mut self.state);

        Ok(())
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_logs(&mut self, logs: Vec<Commit>) {
        self.commits = logs;
    }

    fn reset(&mut self) {
        self.selected_commits.clear();
        self.event_sender
            .send(ProgramEvent::Focus(ComponentType::BranchComponent))
            .expect("Focus event send failed.");
        self.visible = false;
    }

    fn cherry_pick(&mut self) -> Result<(), Error> {
        if let Err(err) = cherry_pick(&self.repo_path, &self.selected_commits) {
            self.event_sender.send(ProgramEvent::Error(err)).expect("Send Failed");
        }
        self.selected_commits.clear();
        Ok(())
    }

    fn select_commit(&mut self) {
        if let Some(commit) = self.commits.get(self.position) {
            let id = commit.id();

            if self.selected_commits.contains(id) {
                let index = self
                    .selected_commits
                    .iter()
                    .position(|c| c == id)
                    .expect("Commit SHA should exist");
                self.selected_commits.remove(index);
            } else {
                self.selected_commits.push(id.to_string());
            }
        }
    }
}

impl Component for CherryPickPopup {
    fn update(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
        if !self.visible {
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
                self.scroll_down(10);
            }
            KeyCode::Char('u') if ev.modifiers == KeyModifiers::CONTROL => {
                self.scroll_up(10);
            }
            KeyCode::Char('s') => self.select_commit(),
            KeyCode::Esc => self.reset(),
            KeyCode::Enter => self.cherry_pick()?,
            _ => {}
        }
        Ok(())
    }

    fn focus(&mut self, focus: bool) {
        self.visible = focus;
    }
}

impl ScrollableComponent for CherryPickPopup {
    fn get_list_length(&self) -> usize {
        self.commits.len()
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
