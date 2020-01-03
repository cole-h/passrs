use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use termion::input::TermRead;

use failure::Fallible;

use crate::util;
use crate::PassrsError;

pub fn insert(force: bool, echo: bool, pass_name: String, secret: Option<String>) -> Fallible<()> {
    let path = util::canonicalize_path(&pass_name)?;
    // let path = format!("{}.gpg", path);

    // TODO: recursively create dir
    // match fs::create_dir_all(&path) {} -- if "Exists", go deeper
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    if !force && util::path_exists(&path)? {
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
                    .mode(0o600)
                    .write(true)
                    .truncate(true)
                    .open(&path)?;
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    // TODO: if pass_name is a folder, write to pass_name/otp
    // TODO: get path from store root

    if force {
        // TODO
        // if path exists, message = "Replace";
    }
    if echo {
        // TODO
    }
    if !force && util::path_exists(&pass_name)? {
        return Err(PassrsError::PathExists(pass_name).into());
    }

    // TODO: insert secret
    let _ = secret;

    util::commit(format!("Add OTP secret for {} to store", pass_name))?;
    Ok(())
}
