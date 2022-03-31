use cursive::Cursive;
use cursive::views::Dialog;
use git2::{Repository, BranchType, Branch, Error};


fn main() {
    let repo = match Repository::open("/Users/reina/school/groupwork/capstone/") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let branches = get_local_branches(&repo);
    let branch_names = get_branch_name(&branches).unwrap();
    //checkout_branch(&repo, branch_names.get(1).unwrap());

    let mut siv = cursive::default();

    siv.add_global_callback('q', |s| s.quit());

    siv.add_layer(Dialog::text(branch_names.join("\n"))
        .title("Important")
        .button("Next", show_next));

    siv.run();
}

#[allow(dead_code)]
fn show_next(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(Dialog::text("Did you do the thing?")
        .title("Question 1")
        .button("Yes!", |s| show_answer(s, "Well done!"))
        .button("No", |s| show_answer(s, "I knew you couldn't be trusted!!!"))
        .button("Uh...?", |s| s.add_layer(Dialog::info("Try again!"))));
}

#[allow(dead_code)]
fn show_answer(s: &mut Cursive, msg: &str) {
    s.pop_layer();
    s.add_layer(Dialog::text(msg)
        .title("Results")
        .button("Finish", |s| s.quit()));
}

fn get_local_branches(repo: &Repository) -> Vec<Branch> {
    let mut local_branches: Vec<Branch> = Vec::new();

    let branches = repo.branches(Some(BranchType::Local)).unwrap();
    let branches: Vec<Result<(Branch, BranchType), Error>> = branches.collect::<Vec<Result<_, _>>>();

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
    let (object, reference) = repo.revparse_ext(refname).expect("Object not found");
    repo.checkout_tree(&object, None).expect("Failed to checkout");

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }.expect("Failed to set HEAD");
}
