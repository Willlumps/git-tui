use crate::components::diff::DiffLine;

use anyhow::Result;
use git2::DiffFormat;
use git2::Repository;
use tui::style::{Color, Style};

use std::path::Path;

pub fn get_diff(repo_path: &Path) -> Result<Vec<DiffLine>> {
    let repo = match Repository::init(repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to init: {}", e),
    };

    let mut diff_lines: Vec<DiffLine> = Vec::new();

    let mut opt = git2::DiffOptions::new();
    let diff = repo.diff_index_to_workdir(None, Some(&mut opt))?;

    diff.print(DiffFormat::Patch, |_d, _h, l| {
        if let Ok(diff_line) = std::str::from_utf8(l.content()) {
            let line_style = match l.origin() {
                '-' => Style::default().fg(Color::Red),
                '+' => Style::default().fg(Color::Green),
                'H' => Style::default().fg(Color::Cyan),
                _ => Style::default(),
            };

            diff_lines.push(DiffLine {
                content: diff_line.to_string(),
                origin: l.origin(),
                style: line_style,
            });
        };
        true
    })?;

    Ok(diff_lines)
}
