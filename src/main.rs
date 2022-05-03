#![allow(unused_imports)]
mod list;
use list::List;

use crossterm::{
    event::{poll, read, DisableMouseCapture, Event as CEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use git2::{Branch, BranchType, Error, Repository};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::{
    io, thread,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List as TuiList, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

enum Event<I> {
    Input(I),
    Tick,
}

struct App {
    //repo: &'a Repository,
    input: String,
    branches: List,
}

impl App {
    //fn new(repo: &'a Repository) -> Self {
    fn new() -> Self {
        Self {
            //repo,
            input: String::new(),
            branches: List::new(),
        }
    }
}

fn main() -> crossterm::Result<()> {
    // let repo = match Repository::open("/Users/reina/school/groupwork/capstone/") {
    //     Ok(repo) => repo,
    //     Err(e) => panic!("failed to open: {}", e),
    // };

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

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app, rx);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let container = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);

    // Status, Files, Branches, Logs?
    let left_container = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Length(20),
                Constraint::Length(10),
            ]
            .as_ref(),
        )
        .split(container[0]);

    let status_container = Paragraph::new(" Placeholder ")
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .border_type(BorderType::Rounded)
                .title(" Status "));
    f.render_widget(status_container, left_container[0]);

    let file_block = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .border_type(BorderType::Rounded);
    f.render_widget(file_block, left_container[1]);

    let branch_border = Block::default()
        .title(" Branches ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .border_type(BorderType::Rounded);
    f.render_widget(branch_border, left_container[2]);

    let log_block = Block::default()
        .title(" Log ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .border_type(BorderType::Rounded);
    f.render_widget(log_block, left_container[3]);

    let branch_container = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
        .split(left_container[2]);

    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .border_type(BorderType::Rounded)
                .title(" Search ")
                .title_alignment(Alignment::Center),
        );

    let list_items: Vec<ListItem> = app
        .branches
        .filtered_branches
        .iter()
        .map(|item| ListItem::new(item.to_string()))
        .collect();
    let list = TuiList::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .border_type(BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(input, branch_container[0]);
    f.render_stateful_widget(list, branch_container[1], &mut app.branches.state);

    // Right Diff
    let diff_block = Block::default()
        .title(" Diff ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .border_type(BorderType::Rounded);
    f.render_widget(diff_block, container[1]);
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: &mut App,
    rx: Receiver<Event<crossterm::event::KeyEvent>>,
) -> io::Result<()> {
    app.branches.state.select(Some(0));

    loop {
        terminal.draw(|f| ui(f, app))?;

        match rx.recv() {
            Ok(event) => match event {
                Event::Input(input) => match input.code {
                    KeyCode::Char('q') if input.modifiers == KeyModifiers::CONTROL => {
                        return Ok(());
                    }
                    KeyCode::Char('j') if input.modifiers == KeyModifiers::CONTROL => {
                        app.branches.decrement_position();
                    }
                    KeyCode::Char('k') if input.modifiers == KeyModifiers::CONTROL => {
                        app.branches.increment_position();
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        app.branches.filtered_branches =
                            fuzzy_find(&app.branches.branches, &app.input);
                        app.branches.reset_state();
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        app.branches.filtered_branches =
                            fuzzy_find(&app.branches.branches, &app.input);
                        app.branches.reset_state();
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
}

fn fuzzy_find(filtered_list: &[String], query: &str) -> Vec<String> {
    let matcher = SkimMatcherV2::default();
    filtered_list
        .iter()
        .filter(|&item| matcher.fuzzy_match(item, query).is_some())
        .cloned()
        .collect::<Vec<_>>()
}
