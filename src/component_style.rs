use crate::git::git_status::StatusLoc;

use tui::style::{Color, Modifier, Style};

pub struct ComponentTheme {
    style: Style,
    border_style: Style,
}

impl ComponentTheme {
    pub fn default() -> Self {
        Self {
            style: Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
            border_style: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::DIM),
        }
    }

    pub fn focused() -> Self {
        Self {
            style: Style::default().fg(Color::White),
            border_style: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        }
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn border_style(&self) -> Style {
        self.border_style
    }

    pub fn file_status_style(loc: StatusLoc) -> Style {
        match loc {
            StatusLoc::Index => Style::default().fg(Color::Green),
            StatusLoc::WorkingDirectory => Style::default().fg(Color::Red),
            StatusLoc::WorkingDirectoryAndIndex => Style::default().fg(Color::Yellow),
            StatusLoc::None => Style::default().fg(Color::White),
        }
    }
}
