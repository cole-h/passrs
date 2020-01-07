// TODO: Mac?

use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use crate::consts::PASSWORD_STORE_X_SELECTION;
use crate::PassrsError;

pub fn clip<S>(contents: S) -> Result<()>
where
    S: AsRef<[u8]>,
{
    let contents = contents.as_ref();
    if env::var("WAYLAND_DISPLAY").is_ok() {
        Command::new("wl-copy")
            .arg("--trim-newline")
            .stdin(Stdio::piped())
            .spawn()
            .with_context(|| "Failed to spawn wl-copy")?
            .stdin
            .with_context(|| "stdin wasn't captured")?
            .write_all(contents)?;
    } else if env::var("DISPLAY").is_ok() {
        Command::new("xclip")
            .args(&["-in", "-selection", &PASSWORD_STORE_X_SELECTION])
            .stdin(Stdio::piped())
            .spawn()
            .with_context(|| "Failed to spawn xclip")?
            .stdin
            .with_context(|| "stdin wasn't captured")?
            .write_all(contents)?;
    } else {
        return Err(PassrsError::ClipFailed.into());
    }

    Ok(())
}

pub fn paste() -> Result<Vec<u8>> {
    let bytes = if env::var("WAYLAND_DISPLAY").is_ok() {
        Command::new("wl-paste")
            .arg("--no-newline")
            .output()
            .with_context(|| "Failed to spawn wl-paste")?
            .stdout
    } else if env::var("DISPLAY").is_ok() {
        Command::new("xclip")
            .args(&["-out", "-selection", &PASSWORD_STORE_X_SELECTION])
            .output()
            .with_context(|| "Failed to spawn xclip")?
            .stdout
    } else {
        return Err(PassrsError::PasteFailed.into());
    };

    Ok(bytes)
}
