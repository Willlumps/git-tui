use std::env::current_dir;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam::channel::{unbounded, Receiver, Select, Sender};
use crossbeam::sync::{Parker, Unparker};
use crossterm::event::{poll, read, DisableMouseCapture, Event as CEvent, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use git::commit::create_initial_commit;
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;

use crate::app::{App, ProgramEvent};
use crate::components::{centered_rect, ComponentType};
use crate::git::{init_new_repo, is_empty_repo, is_repo};
use crate::ui::{main_ui, prompt_new_repo};

mod app;
mod component_style;
mod components;
mod git;
mod list_window;
mod ui;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct InputLock {
    unparker: Unparker,
    lock: Sender<()>,
}

fn main() -> Result<()> {
    let (input_tx, input_rx) = unbounded();
    let (program_event_tx, program_event_rx) = unbounded();
    let (input_lock_tx, input_lock_rx) = unbounded();

    let parker = Parker::new();
    let unparker = parker.unparker().clone();
    let input_lock = InputLock {
        unparker,
        lock: input_lock_tx,
    };

    let tick_rate = Duration::from_millis(1200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();

        loop {
            if input_lock_rx.try_recv().is_ok() {
                parker.park();
            }

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if let Ok(poll) = poll(timeout) {
                if poll {
                    if let CEvent::Key(key) = read().expect("Should read event") {
                        input_tx.send(Event::Input(key)).expect("Should send event");
                    }
                } else if last_tick.elapsed() >= tick_rate && input_tx.send(Event::Tick).is_ok() {
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

    let repo_path = current_dir()?;
    // let repo_path = std::path::PathBuf::from("/Users/reina/rust/programming-rust");
    // let repo_path = std::path::PathBuf::from("/Users/reina/projects/rust/test");

    if !is_repo(&repo_path) && prompt_new_repo(&repo_path, &mut terminal, input_rx.clone()).is_err()
    {
        restore_terminal(&mut terminal)?;
        return Ok(());
    }

    if is_empty_repo(&repo_path)? {
        if let Err(err) = create_initial_commit(&repo_path) {
            eprintln!("Failed to make initial commit: {err:?}");
            return Ok(());
        }
    }

    // Initialize and run
    let mut app = App::new(repo_path, &program_event_tx, input_lock);
    let res = run_app(&mut terminal, &mut app, input_rx, program_event_rx);

    restore_terminal(&mut terminal)?;

    if let Err(err) = res {
        eprintln!("{:?}", err)
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
    input_rx: Receiver<Event<KeyEvent>>,
    event_rx: Receiver<ProgramEvent>,
) -> Result<()> {
    loop {
        app.update()?;

        terminal.draw(|f| {
            if let Err(e) = main_ui(f, app) {
                eprintln!("Draw error: {}", e);
            }
        })?;

        let mut select = Select::new();
        select.recv(&event_rx);
        select.recv(&input_rx);

        let operation = select.select();
        match operation.index() {
            0 => {
                let event = operation.recv(&event_rx).expect("Receive failed");
                match event {
                    ProgramEvent::ClearTerminal => terminal.clear()?,
                    ProgramEvent::Exit => return Ok(()),
                    ProgramEvent::Error(error) => app.display_error(error),
                    ProgramEvent::Focus(component) => app.focus(component),
                    ProgramEvent::Git(git_event) => app.handle_git_event(git_event)?,
                }
            }
            1 => {
                let input_event = operation.recv(&input_rx).expect("Receive failed");
                app.handle_input_event(input_event)?;
            }
            _ => {
                unreachable!();
            }
        }
    }
}
