use std::fs;
use std::os::unix::fs::OpenOptionsExt;

use anyhow::Result;

use crate::consts::PASSWORD_STORE_UMASK;
use crate::util;
use crate::Flags;
use crate::PassrsError;

pub fn rm(secret_name: String, flags: Flags) -> Result<()> {
    let recursive = flags.recursive;
    let force = flags.force;
    let path = util::canonicalize_path(&secret_name)?;

    if !force && util::path_exists(&path)? {
        let prompt = format!("Are you sure you would like to delete {}?", secret_name);

        if util::prompt_yesno(prompt)? {
            if path.is_file() {
                fs::OpenOptions::new()
                    .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
                    .write(true)
                    .truncate(true)
                    .open(&path)?;
            }
        } else {
            return Err(PassrsError::UserAbort.into());
        }
    }

    match fs::metadata(&path) {
        Ok(meta) => {
            if meta.is_dir() {
                if recursive {
                    fs::remove_dir_all(&path)?;
                    util::commit(format!("Remove folder {} from store", secret_name))?;
                } else {
                    return Err(PassrsError::PathIsDir(path.display().to_string()).into());
                }
            } else {
                fs::remove_file(path)?;
                util::commit(format!("Remove entry {} from store", secret_name))?;
            }
        }
        Err(_) => {
            return Err(PassrsError::PathDoesntExist(path.display().to_string()).into());
        }
    }

    Ok(())
}
