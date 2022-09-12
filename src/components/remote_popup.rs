use std::path::PathBuf;

use anyhow::Result;
use crossbeam::channel::Sender;
use crossterm::event::{KeyCode, KeyEvent};
use regex::Regex;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Text};
use tui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};
use tui::Frame;

use crate::app::ProgramEvent;
use crate::components::{centered_rect, Component, ComponentType};
use crate::git::remote::add_remote;

pub struct RemotePopupComponent {
    error_message: String,
    event_sender: Sender<ProgramEvent>,
    input_source: u8,
    prompt_user: bool,
    remote_input: String,
    remote_input_style: Style,
    repo_path: PathBuf,
    state: ListState,
    url_input: String,
    url_input_style: Style,
    visible: bool,
}

impl RemotePopupComponent {
    pub fn new(repo_path: PathBuf, event_sender: Sender<ProgramEvent>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            error_message: String::new(),
            event_sender,
            input_source: 0,
            prompt_user: true,
            remote_input: String::new(),
            remote_input_style: Style::default().fg(Color::Yellow),
            repo_path,
            state,
            url_input: String::new(),
            url_input_style: Style::default().fg(Color::Gray),
            visible: false,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        if self.prompt_user {
            self.draw_prompt(f, rect);
        } else {
            self.draw_form(f, rect);
        }
    }

    fn draw_form<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        let area = centered_rect(50, 11, rect);

        let border = Block::default()
            .title(Span::raw(" Add Remote "))
            .style(Style::default())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let container = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(area);

        let remote_input = Paragraph::new(self.remote_input.as_ref())
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.remote_input_style)
                    .title(" Remote ")
                    .title_alignment(Alignment::Left),
            );

        let url_input = Paragraph::new(self.url_input.as_ref())
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.url_input_style)
                    .title(" URL ")
                    .title_alignment(Alignment::Left),
            );

        let error_message = Paragraph::new(self.error_message.as_ref())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));

        f.render_widget(Clear, area);
        f.render_widget(border, area);

        f.render_widget(remote_input, container[1]);
        f.render_widget(url_input, container[3]);
        f.render_widget(error_message, container[4]);
    }

    fn draw_prompt<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        let area = centered_rect(40, 8, rect);

        let border = Block::default()
            .title(Span::styled(
                " No Remotes Found ",
                Style::default().fg(Color::Red),
            ))
            .style(Style::default())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let container = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(4), Constraint::Length(4)].as_ref())
            .split(area);

        let prompt = Text::raw("\n\nAdd new remote?");

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
        f.render_stateful_widget(list, container[1], &mut self.state);
    }

    fn switch_input_source(&mut self) {
        self.input_source ^= 1;

        if self.input_source == 0 {
            self.remote_input_style = Style::default().fg(Color::Yellow);
            self.url_input_style = Style::default().fg(Color::Gray);
        } else {
            self.remote_input_style = Style::default().fg(Color::Gray);
            self.url_input_style = Style::default().fg(Color::Yellow);
        }
    }

    fn push_input(&mut self, c: char) {
        self.error_message.clear();

        if self.input_source == 0 {
            self.remote_input.push(c);
        } else {
            self.url_input.push(c);
        }
    }

    fn pop_input(&mut self) {
        self.error_message.clear();

        if self.input_source == 0 {
            self.remote_input.pop();
        } else {
            self.url_input.pop();
        }
    }

    fn validate_input(&mut self) -> bool {
        // Wizardry regex source: https://stackoverflow.com/a/63283134
        let valid_url = Regex::new(r"^(([A-Za-z0-9]+@|http(|s)://)|(http(|s)://[A-Za-z0-9]+@))([A-Za-z0-9.]+(:\d+)?)(?::|/)([\d/\w.-]+?)(\.git){1}$")
            .expect("I totally wrote this");
        let invalid_remote_chars = Regex::new(r"[^\w\s\d -]").expect("trust me");

        if !valid_url.is_match(&self.url_input) {
            self.error_message = String::from("Invalid URL");
            return false;
        }

        if invalid_remote_chars.is_match(&self.remote_input) {
            self.error_message = String::from("Invalid remote name");
            return false;
        }

        true
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    fn reset(&mut self) {
        self.event_sender
            .send(ProgramEvent::Focus(ComponentType::FilesComponent))
            .expect("Focus event send failed.");
        self.visible = false;
        self.prompt_user = true;
        self.remote_input.clear();
        self.url_input.clear();
        self.state.select(Some(0));
    }
}

impl Component for RemotePopupComponent {
    fn update(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_event(&mut self, ev: KeyEvent) -> Result<()> {
        match ev.code {
            KeyCode::Char('j') => {
                if self.state.selected() == Some(0) {
                    self.state.select(Some(1))
                }
            }
            KeyCode::Char('k') => {
                if self.state.selected() == Some(1) {
                    self.state.select(Some(0))
                }
            }
            KeyCode::Enter if !self.prompt_user => {
                if self.validate_input() {
                    add_remote(
                        &self.repo_path,
                        self.remote_input.trim(),
                        self.url_input.trim(),
                    )?;
                    self.reset();
                }
            }
            KeyCode::Enter => {
                if let Some(selection) = self.state.selected() {
                    if selection == 0 {
                        self.prompt_user = false;
                    } else {
                        self.reset();
                    }
                }
            }
            KeyCode::Tab => self.switch_input_source(),
            KeyCode::Esc => self.reset(),
            KeyCode::Char(c) => self.push_input(c),
            KeyCode::Backspace => self.pop_input(),
            _ => {}
        }
        Ok(())
    }

    fn focus(&mut self, focus: bool) {
        self.visible = focus;
    }
}
