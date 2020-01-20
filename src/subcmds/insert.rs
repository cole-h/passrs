use std::fs;
use std::os::unix::fs::OpenOptionsExt;

use anyhow::Result;

use crate::consts::PASSWORD_STORE_UMASK;
use crate::util;
use crate::util::EditMode;
use crate::Flags;
use crate::PassrsError;

pub(crate) fn insert(secret_name: String, flags: Flags) -> Result<()> {
    let echo = flags.echo;
    let multiline = flags.multiline;
    let force = flags.force;
    let path = util::canonicalize_path(&secret_name)?;

    util::create_dirs_to_file(&path)?;

    if !force && util::path_exists(&path)? {
        let prompt = format!("An entry exists for {}. Overwrite it?", secret_name);

        if util::prompt_yesno(prompt)? {
            fs::OpenOptions::new()
                .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
                .write(true)
                .truncate(true)
                .open(&path)?;
        } else {
            return Err(PassrsError::UserAbort.into());
        }
    }

    let secret = util::prompt_for_secret(&secret_name, echo, multiline)?;

    // if we prompted the user for a password and got one
    if let Some(secret) = secret {
        util::encrypt_bytes_into_file(secret.as_bytes(), &path, EditMode::Clobber)?;
        util::commit(
            Some([&path]),
            format!("Add given secret for {} to store", secret_name),
        )?;
    }

    Ok(())
}
