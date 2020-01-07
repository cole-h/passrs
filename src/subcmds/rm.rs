use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

use anyhow::Result;
use termion::input::TermRead;

use crate::consts::PASSWORD_STORE_UMASK;
use crate::util;
use crate::PassrsError;

// TODO: `pass rm` also removes the pathspec from the repo
pub fn rm(recursive: bool, force: bool, pass_name: String) -> Result<()> {
    let path = util::canonicalize_path(&pass_name)?;

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if !force && util::path_exists(&path)? {
        write!(
            stdout,
            "Are you sure you would like to delete {}? [y/N] ",
            pass_name
        )?;
        io::stdout().flush()?;

        match stdin.read_line()? {
            Some(reply) if reply.starts_with('y') || reply.starts_with('Y') => {
                if path.is_file() {
                    fs::OpenOptions::new()
                        .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
                        .write(true)
                        .truncate(true)
                        .open(&path)?;
                }
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    match fs::metadata(&path) {
        Ok(meta) => {
            if meta.is_dir() {
                if recursive {
                    fs::remove_dir_all(&path)?;
                    util::commit(format!("Remove folder {} from store", pass_name))?;
                } else {
                    return Err(PassrsError::PathIsDir(path.display().to_string()).into());
                }
            } else {
                fs::remove_file(path)?;
                util::commit(format!("Remove entry {} from store", pass_name))?;
            }
        }
        Err(_) => {
            return Err(PassrsError::PathDoesntExist(path.display().to_string()).into());
        }
    }

    Ok(())
}
