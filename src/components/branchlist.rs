use crate::app::App;

use crossterm::event::{KeyCode, KeyEvent};
use tui::Frame;
use tui::backend::Backend;
use tui::widgets::{Block, Borders, BorderType, List as TuiList, ListItem, ListState, Paragraph};
use tui::style::{Color, Modifier, Style};
use tui::layout::{Alignment, Direction, Layout, Constraint};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use crossterm::{
    event::{poll, read, DisableMouseCapture, Event as CEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};

pub struct BranchComponent {
    pub branches: Vec<String>,
    pub filtered_branches: Vec<String>,
    pub state: ListState,
    pub focused: bool,
    pub size: usize,
    pub position: usize,
    // TODO: Make reusable theme?
    pub style: Style,
    pub input: String,
}

impl BranchComponent {
    // TODO: Don't hardcode the list (obviously)
    pub fn new() -> Self {
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
            style: Style::default().fg(Color::White),
            input: String::new(),
        }
    }

    pub fn draw<B: tui::backend::Backend>(&mut self, f: &mut tui::Frame<B>, rect: tui::layout::Rect,) -> crossterm::Result<()> {
        let branch_block = Block::default()
            .title(" Branches ")
            .borders(Borders::ALL)
            .border_style(self.style)
            .border_type(BorderType::Rounded);
        f.render_widget(branch_block, rect);

        let branch_container = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
            .split(rect);

        let input = Paragraph::new(self.input.as_ref())
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
                    .border_type(BorderType::Rounded)
                    .title(" Search ")
                    .title_alignment(Alignment::Center),
            );

        let list_items: Vec<ListItem> = self
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
        f.render_stateful_widget(list, branch_container[1], &mut self.state);

        Ok(())
    }

    pub fn handle_event(&mut self, ev: KeyEvent) {
        match ev.code {
            KeyCode::Char('j') if ev.modifiers == KeyModifiers::CONTROL => {
                self.decrement_position();
                self.style = Style::default().fg(Color::Blue);
            }
            KeyCode::Char('k') if ev.modifiers == KeyModifiers::CONTROL => {
                self.increment_position();
                self.style = Style::default().fg(Color::Red);
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                self.filtered_branches =
                    fuzzy_find(&self.branches, &self.input);
                self.reset_state();
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.filtered_branches =
                    fuzzy_find(&self.branches, &self.input);
                self.reset_state();
            },
            _ => {}
        }
    }

    pub fn focus(&mut self, focus: bool) {
        self.focused = focus;
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

// TODO: Where to put this
fn fuzzy_find(filtered_list: &[String], query: &str) -> Vec<String> {
    let matcher = SkimMatcherV2::default();
    filtered_list
        .iter()
        .filter(|&item| matcher.fuzzy_match(item, query).is_some())
        .cloned()
        .collect::<Vec<_>>()
}
