use crate::app::{GitEvent, ProgramEvent};
use crate::component_style::ComponentTheme;
use crate::components::Component;
use crate::error::Error;
use crate::git::branch::{
    checkout_local_branch, checkout_remote_branch, delete_branch, get_branches, Branch,
};
use crate::git::fetch::{fetch, pull_head, pull_selected};
use crate::ComponentType;

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossbeam::channel::{unbounded, Sender};
use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState, Tabs};
use tui::Frame;

pub struct BranchComponent {
    branches: Vec<Branch>,
    state: ListState,
    focused: bool,
    focused_tab: usize,
    position: usize,
    style: ComponentTheme,
    repo_path: PathBuf,
    event_sender: Sender<ProgramEvent>,
}

impl BranchComponent {
    pub fn new(repo_path: PathBuf, event_sender: Sender<ProgramEvent>) -> BranchComponent {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            branches: Vec::new(),
            state,
            focused: false,
            focused_tab: 0,
            position: 0,
            style: ComponentTheme::default(),
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

        let titles = ["Local", "Remote"]
            .iter()
            .cloned()
            .map(Spans::from)
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .select(self.focused_tab)
            .highlight_style(Style::default().fg(Color::Yellow));

        let list_items: Vec<ListItem> = self
            .branches
            .iter()
            .map(|branch| {
                let branch = branch.clone();
                let time = String::from(branch.time_since_commit);

                ListItem::new(Spans::from(vec![
                    Span::raw(branch.name),
                    Span::raw(" "),
                    Span::styled(format!("({})", time), Style::default().fg(Color::Yellow)),
                ]))
            })
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

        f.render_widget(tabs, branch_container[0]);
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

    fn tab_left(&mut self) {
        if self.focused_tab > 0 {
            self.focused_tab -= 1;
        }
        self.reset_state();
    }

    fn tab_right(&mut self) {
        if self.focused_tab < 1 {
            self.focused_tab += 1;
        }
        self.reset_state();
    }

    fn reset_state(&mut self) {
        self.position = 0;
        self.state.select(Some(0));
    }
}

impl Component for BranchComponent {
    fn update(&mut self) -> Result<(), Error> {
        self.branches = get_branches(&self.repo_path)?
            .into_iter()
            .filter(|branch| match self.focused_tab {
                0 => branch.branch_type == git2::BranchType::Local,
                1 => branch.branch_type == git2::BranchType::Remote,
                _ => unimplemented!(),
            })
            .collect::<Vec<_>>();

        self.branches
            .sort_by(|a, b| a.time_since_commit.cmp(&b.time_since_commit));

        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) -> Result<(), Error> {
        if !self.focused {
            return Ok(());
        }

        match ev.code {
            KeyCode::Char('j') => {
                self.decrement_position();
            }
            KeyCode::Char('k') => {
                self.increment_position();
            }
            KeyCode::Char('h') => {
                self.tab_left();
                self.update()?;
            }
            KeyCode::Char('l') => {
                self.tab_right();
                self.update()?;
            }
            KeyCode::Char('c') => {
                if let Some(branch) = self.branches.get(self.position) {
                    if branch.branch_type == git2::BranchType::Local {
                        checkout_local_branch(&self.repo_path, &branch.name)?;
                    } else {
                        checkout_remote_branch(&self.repo_path, &branch.name)?;
                    }
                }
            }
            KeyCode::Char('d') => {
                // TODO: Get this working for deleting a remote branch.
                //       In testing (using push), the program seems to hang
                //       for a reason unknown to me currently
                if let Some(branch) = self.branches.get(self.position) {
                    delete_branch(&self.repo_path, &branch.name)?;
                }
            }
            KeyCode::Char('n') => {
                self.event_sender
                    .send(ProgramEvent::Focus(ComponentType::BranchPopupComponent))
                    .expect("Send failed.");
            }
            KeyCode::Char('f') => {
                let (progress_sender, _progress_receiver) = unbounded();
                let repo_path = self.repo_path.clone();
                let event_sender = self.event_sender.clone();

                thread::spawn(move || {
                    // TODO: This is a fugly mess, come up with a better way to handle transfer
                    //       progress other than sleeping like a dummy
                    event_sender
                        .send(ProgramEvent::Focus(ComponentType::MessageComponent(
                            "Fetching...".to_string(),
                        )))
                        .expect("Focus event send failed.");

                    if let Err(err) = fetch(&repo_path, progress_sender) {
                        event_sender
                            .send(ProgramEvent::Error(err))
                            .expect("Push failure event send failed.");
                        return;
                    }

                    thread::sleep(Duration::from_millis(500));
                    event_sender
                        .send(ProgramEvent::Git(GitEvent::FetchSuccess))
                        .expect("Push success event send failed.");
                    thread::sleep(Duration::from_millis(1000));
                    event_sender
                        .send(ProgramEvent::Focus(ComponentType::BranchComponent))
                        .expect("Focus event send failed.");
                });
            }
            KeyCode::Char('P') => {
                let (progress_sender, _progress_receiver) = unbounded();
                if let Some(branch) = self.branches.get(self.position) {
                    if let Err(err) = pull_selected(&self.repo_path, &branch.name, progress_sender)
                    {
                        self.event_sender
                            .send(ProgramEvent::Error(err))
                            .expect("Push failure event send failed.");
                    }
                }
            }
            KeyCode::Char('p') => {
                let (progress_sender, _progress_receiver) = unbounded();
                if let Err(err) = pull_head(&self.repo_path, progress_sender) {
                    self.event_sender
                        .send(ProgramEvent::Error(err))
                        .expect("Push failure event send failed.");
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

#[allow(dead_code)]
fn fuzzy_find(filtered_list: &[String], query: &str) -> Vec<String> {
    let matcher = SkimMatcherV2::default();
    filtered_list
        .iter()
        .filter(|&item| matcher.fuzzy_match(item, query).is_some())
        .cloned()
        .collect::<Vec<_>>()
}
