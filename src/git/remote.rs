use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crossbeam::channel::{unbounded, Sender};
use git2::PushOptions;

use crate::ComponentType;
use crate::app::ProgramEvent;
use crate::error::Error;
use crate::git::branch::set_upstream_branch;
use crate::git::callbacks::create_remote_callbacks;
use crate::git::diff::head;
use crate::git::repo;

pub fn add_remote(repo_path: &Path, name: &str, url: &str) -> Result<(), Error> {
    let repo = repo(repo_path)?;
    repo.remote(name, url)?;

    Ok(())
}

pub fn get_remote(repo_path: &Path) -> Result<Option<String>, Error> {
    let repo = repo(repo_path)?;
    let remotes = repo.remotes()?;

    let remote = match remotes.len() {
        0 => None,
        1 => {
            let name = remotes.get(0).expect("I know you're there");
            Some(name.to_string())
        }
        _ => {
            return Err(Error::Unknown(String::from(
                "Unimplemented: Pushing when multiple remotes are present.",
            )));
        }
    };

    Ok(remote)
}

pub fn push(
    event_sender: Sender<ProgramEvent>,
    repo_path: PathBuf,
    remote: String,
) -> Result<(), Error> {
    let (progress_sender, progress_receiver) = unbounded();

    std::thread::spawn(move || {
        let retry_count = Arc::new(Mutex::new(0));

        event_sender
            .send(ProgramEvent::Focus(ComponentType::MessageComponent(
                "Pushing".to_string(),
            )))
            .expect("Focus event send failed.");

        // TODO: Don't hardcode remote
        if let Err(err) = push_to_remote(
            &repo_path,
            progress_sender,
            remote,
            Arc::clone(&retry_count),
        ) {
            event_sender
                .send(ProgramEvent::Error(err))
                .expect("Push failure event send failed.");
            return;
        }

        loop {
            let count = retry_count.lock().unwrap();
            if *count >= 4 {
                event_sender
                    .send(ProgramEvent::Error(Error::Git(git2::Error::from_str(
                        "Bad Credentials",
                    ))))
                    .expect("Focus event send failed.");

                break;
            }

            let progress = progress_receiver.recv().expect("Receive failed");

            if progress >= 100 {
                event_sender
                    .send(ProgramEvent::Focus(ComponentType::FilesComponent))
                    .expect("Focus event send failed.");
                break;
            }
        }
    });

    Ok(())
}

fn push_to_remote(repo_path: &Path, progress_sender: Sender<usize>, remote: String, retry_count: Arc<Mutex<usize>>) -> Result<(), Error> {
    let repo = repo(repo_path)?;

    let mut remote_ref = repo.find_remote(remote.as_str())?;
    let head = head(repo_path)?;
    let refspec = format!("refs/heads/{}", head);

    let mut options = PushOptions::new();
    let callbacks = create_remote_callbacks(progress_sender, Some(retry_count));
    options.remote_callbacks(callbacks);

    remote_ref.push(&[refspec], Some(&mut options))?;

    set_upstream_branch(repo_path, remote.as_str(), "master")?;

    Ok(())
}
