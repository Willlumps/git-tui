#![allow(unused_imports)]
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
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

struct BranchList {
    branches: Vec<String>,
    filtered_branches: Vec<String>,
    state: ListState,
    focused: bool,
    size: usize,
    position: usize,
}

impl BranchList {
    fn new() -> Self {
        let words = vec![
            "main".to_string(),
            "task/ABK-12-Create-simulated-sensors".to_string(),
            "task/ABK-19-Setup-Communication-IoT-Hub".to_string(),
            "task/ABK-20-IoT-Hub-Msg-Handling-Pi".to_string(),
            "task/ABK-23-Create-Azure-Function-Read-Grow-Chamber".to_string(),
            "task/ABK-24-Create-Azure-Function-Write-Grow-Chamber".to_string(),
            "task/ABK-30-Create-graph-components".to_string(),
            "task/ABK-46-Integrate-backend-with-devices".to_string(),
            "task/abk-11-create-sensor-and-actuator-routines".to_string(),
            "task/abk-17-raspberry-pi-interfacing".to_string(),
            "task/abk-42-create-non-blocking-arduino-routine".to_string(),
            "task/abk-9-create-motr-and-servo-routine".to_string(),
            "topic/ABK-47-Integrate-backend-frontend".to_string(),
        ];

        Self {
            branches: words.clone(),
            filtered_branches: words.clone(),
            state: ListState::default(),
            focused: true,
            size: words.len(),
            position: 0,
        }
    }

    fn set_size(&mut self, size: usize) {
        self.size = size;
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

    fn reset_state(&mut self) {
        self.set_size(self.filtered_branches.len());
        self.position = 0;
        self.state.select(Some(0));
    }
}

enum Event<I> {
    Input(I),
    Tick,
}

struct App {
    //repo: &'a Repository,
    input: String,
    branches: BranchList,
}

impl App {
    //fn new(repo: &'a Repository) -> Self {
    fn new() -> Self {
        Self {
            //repo,
            input: String::new(),
            branches: BranchList::new(),
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
        .split(size);

    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Input"));

    let list_items: Vec<ListItem> = app
        .branches.filtered_branches
        .iter()
        .map(|item| ListItem::new(item.to_string()))
        .collect();
    let list = List::new(list_items)
        .block(Block::default().title("List").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(input, chunks[0]);
    f.render_stateful_widget(list, chunks[1], &mut app.branches.state);
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
                        app.branches.filtered_branches = fuzzy_find(&app.branches.branches, &app.input);
                        app.branches.reset_state();
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        app.branches.filtered_branches = fuzzy_find(&app.branches.branches, &app.input);
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
