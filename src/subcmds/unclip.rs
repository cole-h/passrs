use std::env;
use std::io;
use std::str;
use std::thread;
use std::time;

use anyhow::Result;
use data_encoding::HEXLOWER;
use psutil::process;
use ring::digest;

use crate::clipboard;
use crate::consts::PASSRS_UNCLIP_HASH;
use crate::PassrsError;

pub(crate) fn unclip(timeout: u64, force: bool) -> Result<()> {
    if PASSRS_UNCLIP_HASH.is_empty() {
        eprintln!(
            "Unclip is spawned in the background when you copy to your clipboard.\n\
             You shouldn't call this yourself."
        );
        return Ok(());
    }

    let procs = self::get_procs()?
        .into_iter()
        .filter(|e| e.exe().unwrap_or_default() == env::current_exe().unwrap_or_default())
        .filter(|e| e.pid as u32 != std::process::id());

    for proc in procs {
        assert_eq!(proc.comm, "passrs");
        proc.kill()?;
    }

    let password_bytes = clipboard::paste()?;
    let password = str::from_utf8(&password_bytes)?;
    let password_hash =
        HEXLOWER.encode(digest::digest(&digest::SHA256, password.as_bytes()).as_ref());

    if !(password_hash == *PASSRS_UNCLIP_HASH || force) {
        clipboard::clear()?;
        return Err(PassrsError::HashMismatch.into());
    }

    thread::sleep(time::Duration::from_secs(timeout));
    clipboard::clear()?;

    Ok(())
}

fn get_procs() -> Result<Vec<process::Process>> {
    loop {
        match process::all() {
            Ok(procs) => return Ok(procs),
            Err(why) => {
                if why.kind() != io::ErrorKind::NotFound
                    && why.kind() != io::ErrorKind::PermissionDenied
                {
                    return Err(why.into());
                }
            }
        }
    }
}
