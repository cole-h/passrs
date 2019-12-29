use std::process::Command;

use failure::Fallible;
use ring::digest;

use crate::consts::PASSRS_UNCLIP_HASH;
use crate::error::PassrsError;

pub fn unclip(timeout: u64, force: bool) -> Fallible<()> {
    if *PASSRS_UNCLIP_HASH == "" {
        eprintln!(
            "Unclip is spawned in the background when you copy to your clipboard. \
             This should not be called by a user."
        );
        // TODO: return early only when I'm not debugging
        #[cfg(not(debug_assertions))]
        return Ok(());
    }

    let password_bytes = Command::new("wl-paste")
        .arg("--no-newline")
        .output()?
        .stdout;
    let password = std::str::from_utf8(&password_bytes)?;
    let password_hash = hex::encode(digest::digest(&digest::SHA256, password.as_bytes()));

    if password_hash != *PASSRS_UNCLIP_HASH && !force {
        Command::new("wl-copy").arg("--clear").spawn()?;

        return Err(PassrsError::HashMismatch(password_hash, PASSRS_UNCLIP_HASH.to_owned()).into());
    }

    std::thread::sleep(std::time::Duration::from_secs(timeout));

    // TODO: clipboard utils
    Command::new("wl-copy").arg("--clear").spawn()?;

    Ok(())
}
