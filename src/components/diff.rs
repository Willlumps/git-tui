use crate::git::gitdiff::get_diff;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use git2::DiffLine as Git2DiffLine;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List as TuiList, ListItem, ListState, Paragraph};
use tui::Frame;

pub struct DiffComponent {
    pub diffs: Vec<DiffLine>, // TODO
    pub state: ListState,
    pub focused: bool,
    pub size: usize,
    pub position: usize,
    pub style: Style,
    pub path: String,
}

pub struct DiffLine {
    pub content: String,
    pub origin: char,
    pub style: Style,
}

impl DiffLine {
    pub fn origin(&self) -> char {
        self.origin
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn content(&self) -> &String {
        &self.content
    }
}

impl DiffComponent {
    pub fn new(repo_path: &str) -> Self {
        let diffs = get_diff(repo_path).unwrap();
        let len = diffs.len();

        Self {
            diffs,
            state: ListState::default(),
            focused: false,
            size: len,
            position: 0,
            style: Style::default().fg(Color::White),
            path: repo_path.to_string(),
        }
    }
}

impl DiffComponent {
    pub fn draw<B: tui::backend::Backend>(
        &mut self,
        f: &mut tui::Frame<B>,
        rect: tui::layout::Rect,
    ) -> crossterm::Result<()> {
        self.update_diff();
        let list_items: Vec<ListItem> = self
            .diffs
            .iter()
            .map(|item| {
                let content = match item.origin() {
                    '-' => format!("-{}", item.content()),
                    '+' => format!("+{}", item.content()),
                    _ => item.content().to_string(),
                };
                let text = Span::styled(content, item.style());
                ListItem::new(text)
            })
            .collect();
        let list = TuiList::new(list_items)
            .block(
                Block::default()
                    .title(" Diff ")
                    .borders(Borders::ALL)
                    .border_style(self.style)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_stateful_widget(list, rect, &mut self.state);

        Ok(())
    }

    pub fn update_diff(&mut self) {
        let path = &self.path;
        self.diffs = get_diff(path.as_ref()).unwrap();
    }
}
