use std::io::{self, BufRead, Write};
use termion::input::TermRead;

use failure::Fallible;

use crate::error::PassrsError;
use crate::util;

pub fn insert(echo: bool, multiline: bool, force: bool, pass_name: String) -> Fallible<()> {
    let path = util::canonicalize_path(&pass_name)?;
    let path = format!("{}.gpg", path);

    // TODO: create dirs recursively

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    if util::path_exists(&path).is_err() && !force {
        write!(
            stdout,
            "An entry exists for {}. Overwrite it? [y/N] ",
            pass_name
        )?;
        io::stdout().flush()?;

        match stdin.read_passwd(&mut stdout)? {
            Some(reply)
                if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') =>
            {
                std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&path)?;
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    let password = if echo {
        write!(stdout, "Enter password for {}: ", pass_name)?;
        io::stdout().flush()?;
        let input = stdin.read_passwd(&mut stdout)?;

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
        write!(stdout, "Enter password for {}: ", pass_name)?;
        io::stdout().flush()?;
        let input = {
            let input = stdin.read_passwd(&mut stdout)?;
            if input.is_none() {
                return Err(PassrsError::UserAbort.into());
            }
            input.unwrap()
        };

        write!(stdout, "Re-enter password for {}: ", pass_name)?;
        io::stdout().flush()?;
        let check = {
            let input = stdin.read_passwd(&mut stdout)?;
            if input.is_none() {
                return Err(PassrsError::UserAbort.into());
            }
            input.unwrap()
        };

        if input == check {
            Some(input)
        } else {
            return Err(PassrsError::PasswordsDontMatch.into());
        }
    };

    // if we prompted the user for a password and got one
    if let Some(password) = password {
        util::encrypt_bytes_into_file(password.as_bytes(), path)?;
    }

    util::commit(format!("Add given password for {} to store", pass_name))?;
    Ok(())
}
