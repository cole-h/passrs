use std::fs;
use std::io::{self, Write};
use termion::input::TermRead;

use failure::Fallible;

use crate::error::PassrsError;
use crate::util;

pub fn mv(force: bool, source: String, dest: String) -> Fallible<()> {
    let source_path = util::canonicalize_path(&source)?;
    let dest_path = util::canonicalize_path(&dest)?;

    // TODO: find a better way to determine if the user neglected .gpg or not
    let is_file = match fs::metadata(&source_path) {
        Ok(md) => md.is_file(),
        Err(_) => false,
    } || match fs::metadata(&format!("{}.gpg", source_path)) {
        Ok(md) => md.is_file(),
        Err(_) => false,
    };

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    if is_file {
        let source_path = if source_path.ends_with(".gpg") {
            source_path
        } else {
            format!("{}.gpg", source_path)
        };
        let dest_path = if dest_path.ends_with(".gpg") {
            dest_path
        } else {
            format!("{}.gpg", dest_path)
        };

        if util::path_exists(&source_path).is_ok() {
            return Err(PassrsError::NotInStore(source).into());
        }

        if util::path_exists(&dest_path).is_err() && !force {
            write!(
                stdout,
                "An entry exists for {}. Overwrite it? [y/N] ",
                dest_path
            )?;
            io::stdout().flush()?;

            match stdin.read_passwd(&mut stdout)? {
                Some(reply)
                    if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') =>
                {
                    std::fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(&dest_path)?;
                }
                _ => return Err(PassrsError::UserAbort.into()),
            }
        }

        // Rather than reinventing the wheel, just copy source -> dest and
        // delete source file
        util::copy(&source_path, &dest_path, None)?;
        fs::remove_file(&source_path)?;
    } else {
        if util::path_exists(&source_path).is_ok() {
            return Err(PassrsError::PathDoesntExist(source).into());
        }

        if util::path_exists(&dest_path).is_err() && !force {
            write!(
                stdout,
                "An entry exists for {}. Overwrite it? [y/N] ",
                dest_path
            )?;
            io::stdout().flush()?;

            match stdin.read_passwd(&mut stdout)? {
                Some(reply)
                    if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') =>
                {
                    // `rm -rf` the destination because the user said yes
                    fs::remove_dir_all(&dest_path)?;
                }
                _ => return Err(PassrsError::UserAbort.into()),
            }
        }

        // Rather than reinventing the wheel, recursively copy source -> dest
        // and delete source dir
        util::copy(&source_path, &dest_path, None)?;
        fs::remove_dir_all(&source_path)?;
    }

    util::commit(format!("Rename {} to {}", source, dest))?;
    Ok(())
}
