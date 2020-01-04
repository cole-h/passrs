use std::fs;
use std::io::{self, BufRead, Write};
use std::os::unix::fs::OpenOptionsExt;

use anyhow::Result;
use termion::input::TermRead;

use crate::util;
use crate::util::FileMode;
use crate::PassrsError;

pub fn insert(echo: bool, multiline: bool, force: bool, pass_name: String) -> Result<()> {
    let path = util::canonicalize_path(&pass_name)?;

    util::create_descending_dirs(&path)?;

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if !force && util::path_exists(&path)? {
        write!(
            stdout,
            "An entry exists for {}. Overwrite it? [y/N] ",
            pass_name
        )?;
        io::stdout().flush()?;

        match TermRead::read_line(&mut stdin)? {
            Some(reply)
                if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') =>
            {
                fs::OpenOptions::new()
                    .mode(0o600)
                    .write(true)
                    .truncate(true)
                    .open(&path)?;
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    let password = if echo {
        write!(stdout, "Enter secret for {}: ", pass_name)?;
        io::stdout().flush()?;
        let input = TermRead::read_line(&mut stdin)?;

        if input.is_none() {
            return Err(PassrsError::UserAbort.into());
        }

        input
    } else if multiline {
        writeln!(
            stdout,
            "Enter the contents of {} and press Ctrl-D when finished:\n",
            pass_name
        )?;
        let mut input = Vec::new();

        for line in stdin.lines() {
            input.push(line?);
        }

        Some(input.join("\n"))
    } else {
        write!(stdout, "Enter secret for {}: ", pass_name)?;
        io::stdout().flush()?;
        let input = {
            let input = stdin.read_passwd(&mut stdout)?;
            writeln!(stdout)?;
            if input.is_none() {
                return Err(PassrsError::UserAbort.into());
            }

            input.unwrap()
        };

        write!(stdout, "Re-enter secret for {}: ", pass_name)?;
        io::stdout().flush()?;
        let check = {
            let input = stdin.read_passwd(&mut stdout)?;
            writeln!(stdout)?;
            if input.is_none() {
                return Err(PassrsError::UserAbort.into());
            }

            input.unwrap()
        };

        if input == check {
            Some(input)
        } else {
            return Err(PassrsError::SecretsDontMatch.into());
        }
    };

    // if we prompted the user for a password and got one
    if let Some(password) = password {
        util::encrypt_bytes_into_file(password.as_bytes(), path, FileMode::Clobber)?;
    }

    util::commit(format!("Add given secret for {} to store", pass_name))?;
    Ok(())
}
