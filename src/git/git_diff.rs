use crate::components::diff::DiffLine;
use crate::git::repo;

use std::path::Path;

use anyhow::Result;
use git2::DiffFormat;
use tui::style::{Color, Style};

#[derive(Default)]
pub struct DiffWindow {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub branch: String,
}

pub fn get_diff(repo_path: &Path) -> Result<Vec<DiffLine>> {
    let repo = repo(repo_path)?;

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

pub fn get_diff_stats(repo_path: &Path) -> Result<DiffWindow> {
    let repo = repo(repo_path)?;

    let mut opt = git2::DiffOptions::new();
    let diff = repo.diff_index_to_workdir(None, Some(&mut opt))?;
    let stats = diff.stats()?;
    let status = DiffWindow {
        files_changed: stats.files_changed(),
        insertions: stats.insertions(),
        deletions: stats.deletions(),
        branch: head(repo_path)?,
    };

    Ok(status)
}

pub fn head(repo_path: &Path) -> Result<String, git2::Error> {
    let repo = repo(repo_path)?;
    let head_ref = repo.head()?;
    if let Some(branch_name) = head_ref.shorthand() {
        Ok(String::from(branch_name))
    } else {
        Ok("".to_string())
    }
}
