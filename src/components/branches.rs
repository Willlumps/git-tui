use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossbeam::channel::{unbounded, Sender};
use crossterm::event::{KeyCode, KeyEvent};
use git2::{BranchType, MergeOptions};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState, Tabs};
use tui::Frame;

use crate::app::{GitEvent, ProgramEvent};
use crate::component_style::ComponentTheme;
use crate::components::{Component, ScrollableComponent};
use crate::git::branch::{
    checkout_local_branch, checkout_remote_branch, delete_branch, get_branches, Branch,
};
use crate::git::commit::merge_commit;
use crate::git::fetch::{fetch, pull_head, pull_selected};
use crate::git::log::collect_commits;
use crate::git::repo;
use crate::ComponentType;

pub struct BranchComponent {
    branches: Vec<Branch>,
    event_sender: Sender<ProgramEvent>,
    focused: bool,
    focused_tab: BranchType,
    position: usize,
    repo_path: PathBuf,
    state: ListState,
    style: ComponentTheme,
}

impl BranchComponent {
    pub fn new(repo_path: PathBuf, event_sender: Sender<ProgramEvent>) -> BranchComponent {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            branches: Vec::new(),
            event_sender,
            focused: false,
            focused_tab: BranchType::Local,
            position: 0,
            repo_path,
            state,
            style: ComponentTheme::default(),
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
            .select(match self.focused_tab {
                BranchType::Local => 0,
                BranchType::Remote => 1,
            })
            .highlight_style(Style::default().fg(Color::Yellow));

        let list_items: Vec<ListItem> = self
            .branches
            .iter()
            .map(|branch| {
                let branch = branch.clone();
                let time = String::from(*branch.last_commit.time().time_since_commit());

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

    fn tab(&mut self, tab_type: BranchType) -> Result<()> {
        self.focused_tab = tab_type;
        self.reset_state();
        self.update()?;
        Ok(())
    }

    fn reset_state(&mut self) {
        self.position = 0;
        self.state.select(Some(0));
    }

    fn checkout_branch(&self) -> Result<()> {
        if let Some(branch) = self.branches.get(self.position) {
            if branch.branch_type == git2::BranchType::Local {
                checkout_local_branch(&self.repo_path, &branch.name)?;
            } else {
                checkout_remote_branch(&self.repo_path, &branch.name)?;
            }
        }

        Ok(())
    }

    fn cherry_pick(&self) -> Result<()> {
        if let Some(branch) = self.branches.get(self.position) {
            let commit = &branch.last_commit;
            let oid = git2::Oid::from_str(commit.id())?;
            let commits = collect_commits(&self.repo_path, oid)?;

            self.event_sender
                .send(ProgramEvent::Focus(ComponentType::CherryPickPopup(commits)))
                .expect("Send Failed");
        }
        Ok(())
    }

    fn create_branch(&self) {
        self.event_sender
            .send(ProgramEvent::Focus(ComponentType::BranchPopupComponent))
            .expect("Send failed.");
    }

    fn delete_branch(&self) -> Result<()> {
        // TODO: Get this working for deleting a remote branch.
        //       In testing (using push), the program seems to hang
        //       for a reason unknown to me currently
        if let Some(branch) = self.branches.get(self.position) {
            delete_branch(&self.repo_path, &branch.name)?;
        }

        Ok(())
    }

    fn fetch(&self) -> Result<()> {
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

        Ok(())
    }

    fn merge(&self) -> Result<()> {
        let repo = repo(&self.repo_path)?;
        if let Some(branch) = self.branches.get(self.position) {
            let refname = format!("refs/heads/{}", branch.name);
            match repo.find_reference(&refname) {
                Ok(reference) => {
                    let mut opts = MergeOptions::new();
                    let annotated_commit = repo.reference_to_annotated_commit(&reference)?;

                    repo.merge(
                        &[&annotated_commit],
                        Some(&mut opts),
                        Some(git2::build::CheckoutBuilder::default().force()),
                    )?;

                    if let Err(err) = merge_commit(&self.repo_path, annotated_commit) {
                        self.event_sender
                            .send(ProgramEvent::Error(err))
                            .expect("Send failed");
                    }
                }
                Err(err) => {
                    self.event_sender
                        .send(ProgramEvent::Error(anyhow::Error::from(err)))
                        .expect("Send failed");
                }
            }
        }

        repo.cleanup_state()?;

        Ok(())
    }

    fn pull_selected_branch(&self) {
        let (progress_sender, _progress_receiver) = unbounded();
        if let Some(branch) = self.branches.get(self.position) {
            if let Err(err) = pull_selected(&self.repo_path, &branch.name, progress_sender) {
                self.event_sender
                    .send(ProgramEvent::Error(err))
                    .expect("Push failure event send failed.");
            }
        }
    }

    fn pull_head(&self) {
        let (progress_sender, _progress_receiver) = unbounded();
        if let Err(err) = pull_head(&self.repo_path, progress_sender) {
            self.event_sender
                .send(ProgramEvent::Error(err))
                .expect("Push failure event send failed.");
        }
    }
}

impl Component for BranchComponent {
    fn update(&mut self) -> Result<()> {
        self.branches = get_branches(&self.repo_path)?
            .into_iter()
            .filter(|branch| branch.branch_type == self.focused_tab)
            .collect::<Vec<_>>();

        // Ehh
        self.branches.sort_by(|a, b| {
            a.last_commit
                .time()
                .time_since_commit()
                .cmp(b.last_commit.time().time_since_commit())
        });

        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) -> Result<()> {
        if !self.focused {
            return Ok(());
        }

        match ev.code {
            KeyCode::Char('j') => self.scroll_down(1),
            KeyCode::Char('k') => self.scroll_up(1),
            KeyCode::Char('h') => self.tab(BranchType::Local)?,
            KeyCode::Char('l') => self.tab(BranchType::Remote)?,
            KeyCode::Char('c') => self.checkout_branch()?,
            KeyCode::Char('C') => self.cherry_pick()?,
            KeyCode::Char('d') => self.delete_branch()?,
            KeyCode::Char('f') => self.fetch()?,
            KeyCode::Char('m') => self.merge()?,
            KeyCode::Char('n') => self.create_branch(),
            KeyCode::Char('P') => self.pull_selected_branch(),
            KeyCode::Char('p') => self.pull_head(),
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

impl ScrollableComponent for BranchComponent {
    fn get_list_length(&self) -> usize {
        self.branches.len()
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
