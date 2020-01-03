use anyhow::Context;

use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::consts::PASSWORD_STORE_X_SELECTION;
use crate::PassrsError;
use crate::Result;

pub fn clip<S>(contents: S) -> Result<()>
where
    S: AsRef<[u8]>,
{
    let contents = contents.as_ref();
    // TODO: Mac?
    // TODO: genericize over wl-copy, xclip, pbcopy, etc
    //   ref: https://github.com/atotto/clipboard/blob/e9e854e353882a018e9dc587e3757a8822958941/clipboard_unix.go
    // TODO: check if binary exists
    if env::var("WAYLAND_DISPLAY").is_ok() {
        Command::new("wl-copy")
            .arg("--trim-newline")
            .stdin(Stdio::piped())
            .spawn()?
            .stdin
            .with_context(|| "stdin wasn't captured")?
            .write_all(contents)?;
    } else if env::var("DISPLAY").is_ok() {
        Command::new("xclip")
            .args(&["-in", "-selection", &PASSWORD_STORE_X_SELECTION])
            .stdin(Stdio::piped())
            .spawn()?
            .stdin
            .with_context(|| "stdin wasn't captured")?
            .write_all(contents)?;
    }

    Ok(())
}

pub fn paste() -> Result<Vec<u8>> {
    let bytes = if env::var("WAYLAND_DISPLAY").is_ok() {
        Command::new("wl-paste")
            .arg("--no-newline")
            .output()?
            .stdout
    } else if env::var("DISPLAY").is_ok() {
        Command::new("xclip")
            .args(&["-out", "-selection", &PASSWORD_STORE_X_SELECTION])
            .output()?
            .stdout
    } else {
        return Err(PassrsError::PasteFailed.into());
    };

    Ok(bytes)
}
