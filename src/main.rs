#![allow(unused_imports)]
mod app;
mod components;
mod git;
use crate::app::App;

use crossterm::{
    event::{poll, read, DisableMouseCapture, Event as CEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
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

fn main() -> crossterm::Result<()> {
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

    let mut app = App::new("/Users/reina/school/groupwork/capstone".to_string());
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
                .title(" Status "),
        );
    f.render_widget(status_container, left_container[0]);

    app.branches.draw(f, left_container[2]);
    app.logs.draw(f, left_container[3]);
    app.files.draw(f, left_container[1]);
    app.diff.draw(f, container[1]);
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: Receiver<Event<crossterm::event::KeyEvent>>,
) -> io::Result<()> {
    app.branches.state.select(Some(0));
    app.logs.state.select(Some(0));
    app.branches.focus(true);

    loop {
        terminal.draw(|f| ui(f, app))?;

        match rx.recv() {
            Ok(event) => match event {
                Event::Input(input) => match input.code {
                    KeyCode::Char('q') if input.modifiers == KeyModifiers::CONTROL => {
                        return Ok(());
                    }
                    KeyCode::Char('l') if input.modifiers == KeyModifiers::CONTROL => {
                        app.branches.focus(false);
                        app.logs.focus(true);
                    }
                    KeyCode::Char('b') if input.modifiers == KeyModifiers::CONTROL => {
                        app.branches.focus(true);
                        app.logs.focus(false);
                    }
                    _ => {
                        // Do the stuff
                        app.branches.handle_event(input);
                        app.logs.handle_event(input);
                    }
                },
                Event::Tick => {}
            },
            Err(e) => {
                // TODO
                eprintln!("FIX ME {e}")
            }
        }
    }
}
