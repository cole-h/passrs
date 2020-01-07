use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

use anyhow::Result;
use termion::input::TermRead;

use crate::consts::PASSWORD_STORE_UMASK;
use crate::util;
use crate::util::FileMode;
use crate::PassrsError;

pub fn insert(echo: bool, multiline: bool, force: bool, secret_name: String) -> Result<()> {
    let path = util::canonicalize_path(&secret_name)?;

    util::create_descending_dirs(&path)?;

    if !force && util::path_exists(&path)? {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        write!(
            stdout,
            "An entry exists for {}. Overwrite it? [y/N] ",
            secret_name
        )?;
        io::stdout().flush()?;

        match TermRead::read_line(&mut stdin)? {
            Some(reply) if reply.starts_with('y') || reply.starts_with('Y') => {
                fs::OpenOptions::new()
                    .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
                    .write(true)
                    .truncate(true)
                    .open(&path)?;
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    let secret = util::prompt_for_secret(echo, multiline, &secret_name)?;

    // if we prompted the user for a password and got one
    if let Some(secret) = secret {
        util::encrypt_bytes_into_file(secret.as_bytes(), path, FileMode::Clobber)?;
        util::commit(format!("Add given secret for {} to store", secret_name))?;
    }

    Ok(())
}
