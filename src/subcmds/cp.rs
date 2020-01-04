use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;

use anyhow::Result;
use termion::input::TermRead;

use crate::util;
use crate::PassrsError;

pub fn cp(force: bool, source: String, dest: String) -> Result<()> {
    let source_path = util::canonicalize_path(&source)?;
    let is_file = match fs::metadata(&source_path) {
        Ok(meta) => meta.is_file(),
        Err(_) => false,
    };

    let dest_path = if is_file {
        util::exact_path(&format!("{}.gpg", dest))?
    } else {
        util::exact_path(&dest)?
    };

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if is_file {
        if !util::path_exists(&source_path)? {
            return Err(PassrsError::NotInStore(source).into());
        }

        if !force && util::path_exists(&dest_path)? {
            write!(
                stdout,
                "An entry exists for {}. Overwrite it? [y/N] ",
                dest_path.display()
            )?;
            io::stdout().flush()?;

            match stdin.read_line()? {
                Some(reply)
                    if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') =>
                {
                    fs::OpenOptions::new()
                        .mode(0o600)
                        .write(true)
                        .truncate(true)
                        .open(&dest_path)?;
                }
                _ => return Err(PassrsError::UserAbort.into()),
            }
        }

        // Copy file from source_path to dest_path
        util::copy(&source_path, &dest_path, None)?;
    } else {
        if !util::path_exists(&source_path)? {
            return Err(PassrsError::PathDoesntExist(source).into());
        }

        if !force && util::path_exists(&dest_path)? {
            write!(
                stdout,
                "An entry exists for {}. Overwrite it? [y/N] ",
                dest_path.display()
            )?;
            io::stdout().flush()?;

            match stdin.read_line()? {
                Some(reply)
                    if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') =>
                {
                    fs::remove_dir_all(&dest_path)?;
                }
                _ => return Err(PassrsError::UserAbort.into()),
            }
        }

        // Recursively copy folder from source_path to dest_path
        util::copy(&source_path, &dest_path, None)?;
    }

    util::commit(format!("Copy {} to {}", source, dest))?;
    Ok(())
}
