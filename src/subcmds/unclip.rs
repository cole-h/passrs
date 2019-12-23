use std::process::Command;

use ring::digest;

use crate::consts;

pub fn unclip(timeout: u64, force: bool) {
    if *consts::PASSRS_UNCLIP_HASH == "" {
        eprintln!(
            "Unclip is spawned in the background when you copy to your clipboard. \
             This should not be called by a user."
        );
        // TODO: return early only when I'm not debugging
        #[cfg(not(debug_assertions))]
        return;
    }

    let password_bytes = Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .unwrap()
        .stdout;
    let password = unsafe { std::str::from_utf8_unchecked(&password_bytes) };
    let password_hash = hex::encode(digest::digest(&digest::SHA256, password.as_bytes()));

    if password_hash != *consts::PASSRS_UNCLIP_HASH && !force {
        Command::new("wl-copy").arg("--clear").spawn().unwrap();
        eprintln!(
            "Hashes don't match: {} vs {}",
            password_hash,
            envmnt::get_or("PASSRS_UNCLIP_HASH", ""),
        );
        return;
    }

    std::thread::sleep(std::time::Duration::from_secs(timeout));

    // TODO: clipboard utils
    Command::new("wl-copy").arg("--clear").spawn().unwrap();
}
