use std::fs::{self, ReadDir};

use failure::Fallible;

use crate::consts::PASSWORD_STORE_DIR;
use crate::error::PassrsError;
use crate::util;

pub fn rm(recursive: bool, force: bool, pass_name: String) -> Fallible<()> {
    // TODO: verify this is what I want it to do
    let path = format!("{}/{}.gpg", *PASSWORD_STORE_DIR, pass_name);

    if util::path_exists(&path).is_err() && !force {
        match rprompt::prompt_reply_stdout(&format!(
            "Are you sure you want to delete {}? [y/N] ",
            pass_name
        )) {
            Ok(reply) if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') => {
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    if recursive {
        let sep = path.rfind('/').unwrap_or(0);
        delete_dir_contents(fs::read_dir(&path[..sep])?)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

fn delete_dir_contents(dir: ReadDir) -> Fallible<()> {
    for entry in dir {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        };
    }

    Ok(())
}
