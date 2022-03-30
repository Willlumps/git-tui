use cursive::Cursive;
use cursive::views::Dialog;
use std::process::Command;

fn main() {
    // let output = Command::new("git")
    let output = Command::new("pwd")
        // .arg("branch")
        // .arg("-a")
        .output()
        .expect("failed");
    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // let mut siv = cursive::default();
    //
    // siv.add_global_callback('q', |s| s.quit());
    //
    // siv.add_layer(Dialog::text("This is a survey!\nPress <Next> when you're ready.")
    //     .title("Important")
    //     .button("Next", show_next));
    //
    // siv.run();
}

fn show_next(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(Dialog::text("Did you do the thing?")
        .title("Question 1")
        .button("Yes!", |s| show_answer(s, "Well done!"))
        .button("No", |s| show_answer(s, "I knew you couldn't be trusted!!!"))
        .button("Uh...?", |s| s.add_layer(Dialog::info("Try again!"))));
}

fn show_answer(s: &mut Cursive, msg: &str) {
    s.pop_layer();
    s.add_layer(Dialog::text(msg)
        .title("Results")
        .button("Finish", |s| s.quit()));
}
