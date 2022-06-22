use crate::app::App;
use crate::{Error, Event};
use crate::{init_new_repo, centered_rect};

use std::path::Path;

use anyhow::Result;
use crossbeam::channel::Receiver;
use crossterm::event::{KeyCode, KeyEvent};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Text};
use tui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};

pub fn main_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) -> Result<()> {
    let size = f.size();
    let container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);

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

    let right_container = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(container[1]);

    app.status.draw(f, left_container[0])?;
    app.files.draw(f, left_container[1])?;
    app.branches.draw(f, left_container[2])?;
    app.logs.draw(f, left_container[3])?;
    app.diff.draw(f, right_container[0])?;
    app.diff_staged.draw(f, right_container[1])?;

    if app.is_popup_visible() {
        app.draw_popup(f, size)?;
    }

    Ok(())
}

pub fn prompt_new_repo<B: Backend>(
    repo_path: &Path,
    terminal: &mut Terminal<B>,
    rx: Receiver<Event<KeyEvent>>,
) -> Result<(), Error> {
    let mut state = ListState::default();
    state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let area = centered_rect(40, 8, f.size());

            let border = Block::default()
                .title(Span::styled(" Repo Not Found ", Style::default().fg(Color::Red)))
                .style(Style::default())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);

            let container = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(4), Constraint::Length(4)].as_ref())
                .split(area);

            let mut prompt = Text::raw("Initialize new repo at\n");
            prompt.extend(Text::styled(
                format!("{:?}?", repo_path),
                Style::default().fg(Color::Yellow),
            ));

            let init_prompt = Paragraph::new(prompt)
                .alignment(tui::layout::Alignment::Center)
                .style(Style::default().fg(Color::White));

            let options = vec![ListItem::new("Yes"), ListItem::new("No")];

            let list = List::new(options)
                .highlight_style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            f.render_widget(Clear, area);
            f.render_widget(border, area);
            f.render_widget(init_prompt, container[0]);
            f.render_stateful_widget(list, container[1], &mut state);
        })?;

        let input_event = rx.recv().expect("Failed to receive");
        match input_event {
            Event::Input(input) => {
                match input.code {
                    KeyCode::Char('j') => {
                        if state.selected() == Some(0) {
                            state.select(Some(1))
                        }
                    }
                    KeyCode::Char('k') => {
                        if state.selected() == Some(1) {
                            state.select(Some(0))
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(selection) = state.selected() {
                            if selection == 0 {
                                // init repo
                                init_new_repo(repo_path)?;
                                break;
                            } else {
                                return Err(Error::Unknown("NO".to_string()));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Tick => {}
        }
    }
    Ok(())
}
