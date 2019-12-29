use failure::Fallible;

use crate::consts::PASSWORD_STORE_DIR;
use crate::error::PassrsError;
use crate::util;

pub fn insert(
    force: bool,
    echo: bool,
    pass_name: String,
    secret: Option<String>,
) -> Fallible<String> {
    let path = format!("{}/{}.gpg", *PASSWORD_STORE_DIR, pass_name);

    if util::path_exists(&path).is_err() && !force {
        match rprompt::prompt_reply_stdout(&format!(
            "An entry exists for {}. Overwrite it? [y/N] ",
            pass_name
        )) {
            Ok(reply) if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') => {
                std::fs::OpenOptions::new()
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
    if !force && util::path_exists(&pass_name).is_err() {
        return Err(PassrsError::PathExists(pass_name).into());
    }

    // TODO: insert secret
    let _ = secret;

    let message = format!("Add OTP secret for {} to store", pass_name);
    Ok(message)
}
