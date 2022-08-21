use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;
use git2::Cred;
use git2::RemoteCallbacks;

pub fn create_remote_callbacks(
    progress_sender: Sender<usize>,
    retry_count: Option<Arc<Mutex<usize>>>,
) -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();

    // Monitor this. I haven't run across the failed credential loop in a while.
    // Hopefully the `retry_count` will work in breaking out if it does.
    callbacks.credentials(move |_url, username_from_url, allowed_types| {
        let cred: Result<Cred, git2::Error>;

        if allowed_types.is_ssh_key() {
            match username_from_url {
                Some(username) => {
                    cred = Cred::ssh_key_from_agent(username);
                }
                None => {
                    cred = Err(git2::Error::from_str("Where da username??"));
                }
            }
        } else if allowed_types.is_user_pass_plaintext() {
            // Do people actually use plaintext user/pass ??
            unimplemented!();
        } else {
            cred = Cred::default();
        }

        if let Some(count) = &retry_count {
            let mut count = count.lock().unwrap();
            *count += 1;
        }

        cred
    });

    callbacks.push_transfer_progress(move |current, total, _bytes| {
        if let Some(percentage) = current.checked_div(total) {
            progress_sender.send(percentage * 100).expect("Send failed");
        } else {
            progress_sender.send(100).expect("Send failed");
        }
    });

    callbacks.push_update_reference(|_remote, _status| {
        // TODO
        if _status.is_some() {
            panic!("oh no {}", _status.unwrap());
        }
        Ok(())
    });

    callbacks
}
