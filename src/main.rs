mod app;
mod component_style;
mod components;
mod git;
mod list_window;
use crate::app::App;
use components::{Component, ComponentType};

use anyhow::Result;
use crossterm::{
    event::{poll, read, DisableMouseCapture, Event as CEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use std::env::current_dir;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::{
    io, thread,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(500);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if let Ok(poll) = poll(timeout) {
                if poll {
                    if let CEvent::Key(key) = read().expect("Should read event") {
                        tx.send(Event::Input(key)).expect("Should send event");
                    }
                } else if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                    last_tick = Instant::now();
                }
            } else {
                // TODO: Handle Err
            }
        }
    });

    // setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Grab the project root for dev purposes, this will eventually want to be
    // replaced with a passed argument or the current dir where the program
    // is executed from.
    let repo_path = current_dir()?;
    let mut app = App::new(repo_path);
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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) -> Result<()> {
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
                Constraint::Length(4),
                Constraint::Length(8),
                Constraint::Length(15),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(container[0]);

    app.update()?;

    app.status.draw(f, left_container[0])?;
    app.branches.draw(f, left_container[2])?;
    app.logs.draw(f, left_container[3])?;
    app.files.draw(f, left_container[1])?;
    app.diff.draw(f, container[1])?;

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: Receiver<Event<crossterm::event::KeyEvent>>,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            if let Err(e) = ui(f, app) {
                eprintln!("Draw error: {}", e);
            }
        })?;

        match rx.recv() {
            Ok(event) => match event {
                Event::Input(input) => match input.code {
                    KeyCode::Char('q') if input.modifiers == KeyModifiers::CONTROL => {
                        return Ok(());
                    }
                    KeyCode::Char('l') if input.modifiers == KeyModifiers::CONTROL => {
                        app.focus(ComponentType::LogComponent);
                    }
                    KeyCode::Char('b') if input.modifiers == KeyModifiers::CONTROL => {
                        app.focus(ComponentType::BranchComponent);
                    }
                    KeyCode::Char('g') if input.modifiers == KeyModifiers::CONTROL => {
                        app.focus(ComponentType::DiffComponent);
                    }
                    KeyCode::Char('f') if input.modifiers == KeyModifiers::CONTROL => {
                        app.focus(ComponentType::FilesComponent);
                    }
                    _ => {
                        // Do the stuff...poorly
                        app.branches.handle_event(input)?;
                        app.logs.handle_event(input)?;
                        app.diff.handle_event(input)?;
                        app.files.handle_event(input)?;
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
