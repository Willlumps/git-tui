mod app;
mod component_style;
mod components;
mod error;
mod git;
mod list_window;
mod ui;

use crate::app::{App, ProgramEvent};
use crate::components::{centered_rect, ComponentType};
use crate::error::Error;
use crate::git::{init_new_repo, is_empty_repo, is_repo};
use crate::ui::{main_ui, prompt_new_repo};

use std::env::current_dir;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam::channel::{unbounded, Receiver, Select};
use crossterm::event::{
    poll, read, DisableMouseCapture, Event as CEvent, KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use git::commit::create_initial_commit;
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;

pub enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<()> {
    let (tx, rx) = unbounded();
    let (ev_tx, ev_rx) = unbounded();
    let tick_rate = Duration::from_millis(2000);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if let Ok(poll) = poll(timeout) {
                if poll {
                    if let CEvent::Key(key) = read().expect("Should read event") {
                        tx.send(Event::Input(key)).expect("Should send event");
                    }
                } else if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                    last_tick = Instant::now();
                }
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
    //let repo_path = std::path::PathBuf::from("/Users/reina/rust/programming-rust");
    //let repo_path = std::path::PathBuf::from("/Users/reina/projects/rust/test");

    #[allow(clippy::collapsible_if)]
    if !is_repo(&repo_path) {
        if prompt_new_repo(&repo_path, &mut terminal, rx.clone()).is_err() {
            restore_terminal(&mut terminal)?;
            return Ok(());
        }
    }

    if is_empty_repo(&repo_path)? {
        if let Err(err) = create_initial_commit(&repo_path) {
            eprintln!("Failed to make initial commit: {err:?}");
            return Ok(());
        }
    }

    // Initialize and run
    let mut app = App::new(repo_path, &ev_tx);
    let res = run_app(&mut terminal, &mut app, rx, ev_rx);

    restore_terminal(&mut terminal)?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn restore_terminal<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    io::stdout().execute(DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: Receiver<Event<KeyEvent>>,
    event_rx: Receiver<ProgramEvent>,
) -> Result<(), Error> {
    loop {
        app.update()?;

        terminal.draw(|f| {
            if let Err(e) = main_ui(f, app) {
                eprintln!("Draw error: {}", e);
            }
        })?;

        let mut select = Select::new();
        select.recv(&event_rx);
        select.recv(&rx);

        let oper = select.select();
        match oper.index() {
            0 => {
                let event = oper.recv(&event_rx).expect("Receive failed");
                match event {
                    ProgramEvent::Error(error) => {
                        app.display_error(error);
                    }
                    ProgramEvent::Focus(component) => {
                        app.focus(component);
                    }
                    ProgramEvent::Git(git_event) => {
                        app.handle_git_event(git_event)?;
                    }
                }
            }
            1 => {
                let input_event = oper.recv(&rx).expect("Receive failed");
                if app.is_popup_visible() {
                    app.handle_popup_input(input_event);
                } else {
                    match input_event {
                        Event::Input(input) => match input.code {
                            KeyCode::Char('q') if input.modifiers == KeyModifiers::CONTROL => {
                                return Ok(());
                            }
                            KeyCode::Char('1') => {
                                app.focus(ComponentType::FilesComponent);
                            }
                            KeyCode::Char('2') => {
                                app.focus(ComponentType::BranchComponent);
                            }
                            KeyCode::Char('3') => {
                                app.focus(ComponentType::LogComponent);
                            }
                            KeyCode::Char('4') => {
                                app.focus(ComponentType::DiffComponent);
                            }
                            KeyCode::Char('5') => {
                                app.focus(ComponentType::DiffStagedComponent);
                            }
                            _ => {
                                app.handle_input(input);
                            }
                        },
                        Event::Tick => {}
                    }
                }
            }
            _ => {
                unreachable!();
            }
        }
    }
}
