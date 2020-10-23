//! Clipboard helpers
//!
//! # clipboard
//!
//! This module houses the clipboard functionality, utilizing platform-specific
//! binaries to interact with the clipboard.
//!
//! Currently, only Wayland and X11 are supported.

use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time;

use data_encoding::HEXLOWER;
use ring::digest;

use crate::consts::{PASSWORD_STORE_CLIP_TIME, PASSWORD_STORE_X_SELECTION};
use crate::{PassrsError, Result};

/// Copies the `contents` to the clipboard, optionally `force`fully.
///
/// On Wayland, this uses `wl-copy`, and on X11, `xclip`.
pub fn clip<S>(contents: S, force: bool) -> Result<()>
where
    S: AsRef<[u8]>,
{
    let contents = contents.as_ref();

    if env::var("WAYLAND_DISPLAY").is_ok() {
        Command::new("wl-copy")
            .arg("--trim-newline")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn wl-copy: {}", e))?
            .stdin
            .ok_or("stdin wasn't captured")?
            .write_all(contents)?;
    } else if env::var("DISPLAY").is_ok() {
        Command::new("xclip")
            .args(&["-in", "-selection", &PASSWORD_STORE_X_SELECTION])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn xclip: {}", e))?
            .stdin
            .ok_or("stdin wasn't captured")?
            .write_all(contents)?;
    } else {
        return Err(PassrsError::ClipFailed.into());
    }

    let hash = HEXLOWER.encode(digest::digest(&digest::SHA256, &contents).as_ref());
    let args = [
        "unclip",
        &*PASSWORD_STORE_CLIP_TIME,
        if force { "--force" } else { "--" },
    ];

    // Otherwise, the process doesn't live long enough to spawn the unclip
    // daemon
    thread::sleep(time::Duration::from_millis(100));
    Command::new(env::current_exe()?)
        .args(&args)
        .env("PASSRS_UNCLIP_HASH", hash)
        .spawn()?;

    Ok(())
}

/// Retrieves the contents of the clipboard as a `Vec<u8>`.
///
/// On Wayland, this uses `wl-paste`, and on X11, `xclip`.
pub fn paste() -> Result<Vec<u8>> {
    let bytes = if env::var("WAYLAND_DISPLAY").is_ok() {
        Command::new("wl-paste")
            .arg("--no-newline")
            .output()
            .map_err(|e| format!("Failed to spawn wl-paste: {}", e))?
            .stdout
    } else if env::var("DISPLAY").is_ok() {
        Command::new("xclip")
            .args(&["-out", "-selection", &PASSWORD_STORE_X_SELECTION])
            .output()
            .map_err(|e| format!("Failed to spawn xclip: {}", e))?
            .stdout
    } else {
        return Err(PassrsError::PasteFailed.into());
    };

    Ok(bytes)
}

/// Clears the contents of the clipboard.
pub fn clear() -> Result<()> {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        Command::new("wl-copy").arg("--clear").spawn()?;
    } else if env::var("DISPLAY").is_ok() {
        Command::new("xclip")
            .args(&["-in", "-selection", &PASSWORD_STORE_X_SELECTION])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn xclip: {}", e))?
            .stdin
            .ok_or("stdin wasn't captured")?
            .write_all(b"")?;
    } else {
        // unsupported system
    }

    Ok(())
}
