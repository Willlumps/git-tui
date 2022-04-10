//#![allow(unused_imports)]
use crossterm::{
    event::{poll, read, DisableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use git2::{Branch, BranchType, Error, Repository};
use std::sync::mpsc;
use std::{
    io, thread,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> crossterm::Result<()> {
    let repo = match Repository::open("/Users/reina/school/groupwork/capstone/") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let branches = get_local_branches(&repo);

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(500);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if poll(timeout).is_ok() {
                if let CEvent::Key(key) = read().expect("Should read event") {
                    tx.send(Event::Input(key)).expect("Should send event");
                }
            }

            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    // setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut branch_state = ListState::default();
    branch_state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        //Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let header = Paragraph::new("Git Buddy")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));

            let branches = render_branches(&repo);
            f.render_widget(header, chunks[0]);
            f.render_stateful_widget(branches, chunks[1], &mut branch_state);
        })?;

        match rx.recv() {
            Ok(event) => match event {
                Event::Input(input) => match input.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('j') => {
                        if let Some(selected) = branch_state.selected() {
                            if selected < branches.len() - 1 {
                                branch_state.select(Some(selected + 1));
                            }
                        }
                    }
                    KeyCode::Char('k') => {
                        if let Some(selected) = branch_state.selected() {
                            if selected > 0 {
                                branch_state.select(Some(selected - 1));
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = branch_state.selected() {
                            let branch_name = get_branch_name(&branches, selected as usize);
                            checkout_branch(&repo, branch_name);
                        }
                    }
                    _ => {}
                },
                Event::Tick => {}
            },
            Err(e) => {
                eprintln!("FIX ME {e}")
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn render_branches<'a>(repo: &Repository) -> List<'a> {
    let branches = get_local_branches(repo);

    let items: Vec<ListItem> = branches
        .iter()
        .map(|branch| {
            let b = branch.name().expect("Branch should exist");
            if branch.is_head() {
                return ListItem::new(format!("* {}", b.expect("Branch should have a name")))
                    .style(Style::default().fg(Color::Red));
            }
            ListItem::new(b.expect("Branch should have a name").to_string())
        })
        .collect();

    let branches = List::new(items)
        .block(Block::default().title("Branches").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    branches
}

fn get_local_branches(repo: &Repository) -> Vec<Branch> {
    let mut local_branches: Vec<Branch> = Vec::new();

    let branches = repo.branches(Some(BranchType::Local)).unwrap();
    let branches: Vec<Result<(Branch, BranchType), Error>> =
        branches.collect::<Vec<Result<_, _>>>();

    for branch in branches {
        match branch {
            Ok(b) => local_branches.push(b.0),
            Err(e) => eprintln!("{}", e),
        }
    }

    local_branches
}

fn get_branch_name<'a>(branches: &'a [Branch], index: usize) -> &'a str {
    let branch = branches.get(index).expect("Branch should exist");
    let name = branch
        .name()
        .expect("Branch should exist")
        .expect("Branch should have a nanme");
    name
}

fn checkout_branch(repo: &Repository, refname: &str) {
    // Need to change the files in the working directory as well as set the HEAD
    let (object, reference) = repo.revparse_ext(refname).expect("Object not found");
    repo.checkout_tree(&object, None)
        .expect("Failed to checkout");

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }
    .expect("Failed to set HEAD");
}
