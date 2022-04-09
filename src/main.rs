//#![allow(unused_imports)]
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    terminal,
    terminal::ClearType,
    ExecutableCommand,
};
use git2::{Branch, BranchType, Error, Repository};
use std::io;
use std::io::stdout;

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Failed to disable Raw Mode.");
    }
}

fn main() -> Result<(), io::Error> {
    let _clean_up = Cleanup;

    let repo = match Repository::open("/Users/reina/school/groupwork/capstone/") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let branches = get_local_branches(&repo);
    let branch_names = get_branch_name(&branches).unwrap();

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout
        .execute(terminal::Clear(ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?;
    print_branches(&branch_names);
    if let Some(pos) = print_events(&stdout)? {
        let branch = branch_names.get(pos as usize).unwrap();
        checkout_branch(&repo, branch);
        stdout
            .execute(terminal::Clear(ClearType::All))?
            .execute(cursor::MoveTo(0, 0))?;
        println!("Switched to branch: {branch}\r");
    };
    terminal::disable_raw_mode().expect("Failed to disable Raw Mode.");

    Ok(())
}

fn print_branches(branches: &[&str]) {
    for branch in branches {
        println!("{branch}\r");
    }
}

fn print_events(mut stdout: &std::io::Stdout) -> crossterm::Result<Option<u16>> {
    loop {
        match read()? {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('j') => {
                        stdout.execute(cursor::MoveDown(1))?;
                    }
                    KeyCode::Char('k') => {
                        stdout.execute(cursor::MoveUp(1))?;
                    }
                    KeyCode::Enter => {
                        let pos = cursor::position().unwrap();
                        return Ok(Some(pos.1));
                    }
                    _ => {}
                };
            }
            Event::Mouse(event) => println!("{:?}", event),
            Event::Resize(width, height) => println!("New Size {}x{}", width, height),
        }
    }
    Ok(None)
}

fn get_local_branches(repo: &Repository) -> Vec<Branch> {
    let mut local_branches: Vec<Branch> = Vec::new();

    let branches = repo.branches(Some(BranchType::Local)).unwrap();
    let branches: Vec<Result<(Branch, BranchType), Error>> =
        branches.collect::<Vec<Result<_, _>>>();

    for branch in branches {
        match branch {
            Ok(b) => local_branches.push(b.0),
            Err(e) => eprintln!("{}", e),
        }
        // TODO: Possibly only return the name of the branch, though that limits what I'll be able
        //       to do in the future.
        // match branch?.0.name()? {
        //     Some(b) => local_branches.push(b),
        //     None => (),
        // }
    }

    local_branches
}

fn get_branch_name<'a>(branches: &'a [Branch]) -> Result<Vec<&'a str>, Error> {
    let mut branch_names: Vec<&str> = Vec::new();

    for branch in branches {
        if let Some(b) = branch.name()? {
            branch_names.push(b);
        }
    }

    Ok(branch_names)
}

fn checkout_branch(repo: &Repository, refname: &str) {
    // Need to change the files in the working directory as well as set the HEAD
    let (object, reference) = repo.revparse_ext(refname).expect("Object not found");
    repo.checkout_tree(&object, None)
        .expect("Failed to checkout");

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }
    .expect("Failed to set HEAD");
}
