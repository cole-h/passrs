use std::env;
use std::io::{self, Write};
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
        writeln!(
            io::stderr(),
            "Unclip is spawned in the background when you copy to your clipboard.\n\
             You shouldn't call this yourself."
        )?;
        return Ok(());
    }

    let procs = process::processes()?
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.exe().unwrap_or_default() == env::current_exe().unwrap_or_default())
        .filter(|e| e.pid() as u32 != std::process::id());

    for proc in procs {
        assert_ne!(proc.pid() as u32, std::process::id());
        assert_eq!(proc.name()?, "passrs");

        if let Ok(Some(cmdline)) = proc.cmdline() {
            if cmdline.contains("unclip") {
                proc.kill()?;
            }
        }
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
