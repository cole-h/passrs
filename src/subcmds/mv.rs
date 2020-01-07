use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

use anyhow::Result;
use termion::input::TermRead;

use crate::consts::PASSWORD_STORE_UMASK;
use crate::util;
use crate::PassrsError;

pub fn mv(force: bool, source: String, dest: String) -> Result<()> {
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
                Some(reply) if reply.starts_with('y') || reply.starts_with('Y') => {
                    fs::OpenOptions::new()
                        .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
                        .write(true)
                        .truncate(true)
                        .open(&dest_path)?;
                }
                _ => return Err(PassrsError::UserAbort.into()),
            }
        }

        // Copy file from source_path to dest_path
        util::copy(&source_path, &dest_path, None)?;
        fs::remove_dir_all(&source_path)?;
        util::commit(format!("Rename {} to {}", source, dest))?;
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
                Some(reply) if reply.starts_with('y') || reply.starts_with('Y') => {
                    // destination is a dir, `rm -rf` it
                    fs::remove_dir_all(&dest_path)?;
                }
                _ => return Err(PassrsError::UserAbort.into()),
            }
        }

        // Recursively copy folder from source_path to dest_path
        util::copy(&source_path, &dest_path, None)?;
        fs::remove_dir_all(&source_path)?;
        util::commit(format!("Rename {} to {}", source, dest))?;
    }

    Ok(())
}
