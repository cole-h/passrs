use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use termion::input::TermRead;

use failure::Fallible;

use crate::util;
use crate::PassrsError;

// TODO: `pass rm` also removes the pathspec from the repo
pub fn rm(recursive: bool, force: bool, pass_name: String) -> Fallible<()> {
    let path = util::canonicalize_path(&pass_name)?;

    // let is_file = match fs::metadata(&path) {
    //     Ok(md) => md.is_file(),
    //     Err(_) => false,
    // };

    // let path = if path.ends_with(".gpg") {
    //     path
    // } else if !is_file {
    //     format!("{}/", path)
    // } else {
    //     format!("{}.gpg", path)
    // };

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    if !force && util::path_exists(&path)? {
        write!(
            stdout,
            "Are you sure you would like to delete {}? [y/N] ",
            pass_name
        )?;
        io::stdout().flush()?;

        match stdin.read_line()? {
            Some(reply)
                if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') =>
            {
                if path.is_file() {
                    std::fs::OpenOptions::new()
                        .mode(0o600)
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
                    // let sep = path.rfind('/').unwrap_or(0);
                    // fs::remove_dir_all(&path[..sep])?;
                    dbg!(("would remove", &path));
                    fs::remove_dir_all(&path)?;
                } else {
                    return Err(PassrsError::PathIsDir(path.display().to_string()).into());
                }
            } else {
                // fs::remove_file(path)?;
                dbg!(("would remove", path));
            }
        }
        Err(_) => {
            return Err(PassrsError::PathDoesntExist(path.display().to_string()).into());
        }
    }

    util::commit(format!("Remove {} from store", pass_name))?;
    Ok(())
}
