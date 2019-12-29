use failure::Fallible;

use crate::consts::PASSWORD_STORE_DIR;
use crate::error::PassrsError;
use crate::util;

pub fn cp(force: bool, old: String, new: String) -> Fallible<String> {
    let old_path = format!("{}/{}.gpg", *PASSWORD_STORE_DIR, old);
    let new_path = format!("{}/{}.gpg", *PASSWORD_STORE_DIR, new);

    if util::path_exists(&old_path).is_err() && !force {
        match rprompt::prompt_reply_stdout(&format!(
            "An entry exists for {}. Overwrite it? [y/N] ",
            old_path
        )) {
            Ok(reply) if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') => {
                std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&old_path)?;
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    if util::path_exists(&new_path).is_err() && !force {
        match rprompt::prompt_reply_stdout(&format!(
            "An entry exists for {}. Overwrite it? [y/N] ",
            new_path
        )) {
            Ok(reply) if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') => {
                std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&new_path)?;
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    if force {
        //
    }

    let message = format!("Copy {} to {}", old, new);
    Ok(message)
}
