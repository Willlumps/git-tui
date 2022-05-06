fn select_branch(state: &BranchState, repo: &Repository) {
    let branch_list = get_branches(repo, BranchType::Local);
    let index = state.local.selected().expect("Should have state");
    let branch_name = get_branch_name_by_index(&branch_list, index);

    match checkout_branch(repo, &branch_name) {
        Ok(_) => {},
        Err(e) => { eprintln!("Failed to checkout branch: {e}"); },
    };
}

fn move_down(state: &mut BranchState, repo: &Repository) {
    let branch_list = get_branches(repo, BranchType::Local);
    if let Some(selected) = state.local.selected() {
        if selected < branch_list.len() - 1 {
            state.local.select(Some(selected + 1));
        }
    }
}

fn move_up(state: &mut BranchState) {
    if let Some(selected) = state.local.selected() {
        if selected > 0 {
            state.local.select(Some(selected - 1));
        }
    }
}

fn render_branches(repo: &Repository) -> (List, List) {
    let local_branches = render_local_branches(get_branches(repo, BranchType::Local));
    let remote_branches = render_remote_branches(get_branches(repo, BranchType::Remote));

    let local_branch_list = List::new(local_branches)
        .block(Block::default().title("Local").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    let remote_branch_list = List::new(remote_branches)
        .block(Block::default().title("Remote").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    (local_branch_list, remote_branch_list)
}

fn render_local_branches(branches: Vec<Branch>) -> Vec<ListItem> {
    branches
        .iter()
        .map(|branch| {
            let b = branch.name()
                .expect("Branch should exist")
                .expect("Branch should have a name");
            if branch.is_head() {
                return ListItem::new(format!("* {}", b))
                    .style(Style::default().fg(Color::Green));
            }
            ListItem::new(format!("  {}", b))
        })
        .collect()
}

fn render_remote_branches(branches: Vec<Branch>) -> Vec<ListItem> {
    branches
        .iter()
        .map(|branch| {
            let b = branch.name()
                .expect("Branch should exist")
                .expect("Branch should have a name");
                return ListItem::new(format!("  {}", b))
                    .style(Style::default().fg(Color::Red));
        })
        .collect()
}

fn get_branches(repo: &Repository, branch_type: BranchType) -> Vec<Branch> {
    let mut branch_list: Vec<Branch> = Vec::new();

    let branches = repo.branches(Some(branch_type)).unwrap();
    let branches: Vec<Result<(Branch, BranchType), Error>> =
        branches.collect::<Vec<Result<_, _>>>();

    for branch in branches {
        match branch {
            Ok(b) => branch_list.push(b.0),
            Err(e) => eprintln!("{}", e),
        }
    }

    branch_list
}


fn get_branch_name_by_index(branches: &[Branch], index: usize) -> String {
    let branch = branches.get(index).expect("Branch should exist");
    get_branch_name(branch)
}

fn get_branch_name(branch: &Branch) -> String {
    let name = branch
        .name()
        .expect("Branch should exist")
        .expect("Branch should have a nanme");
    name.to_string()
}

fn new_branch<'a>(repo: &'a Repository, refname: &str)  -> Branch<'a> {
    // TODO: Check if reference already exists
    let (object, _reference) = repo.revparse_ext(refname).expect("Object not found");
    let commit = object.as_commit().unwrap();
    repo.branch(&refname[7..], commit, false).unwrap()
}

fn checkout_branch(repo: &Repository, refname: &str) -> Result<(), git2::Error> {
    // TODO: Determine full refname before this point?
    let full_ref = format!("refs/heads/{refname}");
    let cur_ref = repo.head()?;
    let statuses = repo.statuses(Some(git2::StatusOptions::new().include_ignored(false)))?;

    if statuses.is_empty() {
        repo.set_head(&full_ref)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        if let Err(e) = repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force(),)) {
          repo.set_head(cur_ref.name().unwrap())?;
          return Err(e);
       }
       return Ok(());
    } else {
        // Handle uncommitted changes
    }
    Ok(())
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
