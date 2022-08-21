use crossbeam::channel::Sender;
use git2::Cred;
use git2::RemoteCallbacks;

pub fn create_remote_callbacks(progress_sender: Sender<usize>) -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();

    // TODO: This sometimes fails credential check and loop indefinitely
    callbacks.credentials(|_url, username_from_url, allowed_types| {
        if allowed_types.is_ssh_key() {
            match username_from_url {
                Some(username) => Cred::ssh_key_from_agent(username),
                None => Err(git2::Error::from_str("Where da username??")),
            }
        } else if allowed_types.is_user_pass_plaintext() {
            // Do people actually use plaintext user/pass ??
            unimplemented!();
        } else {
            Cred::default()
        }
    });

    callbacks.push_transfer_progress(move |current, total, _bytes| {
        if let Some(percentage) = current.checked_div(total) {
            progress_sender
                .send(percentage * 100)
                .expect("Send failed");
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
