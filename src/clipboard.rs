use std::io::Write;
use std::process::{Command, Stdio};

use failure::{err_msg, Fallible};

pub fn clip(contents: &str) -> Fallible<()> {
    // TODO: Wayland? X11? Mac? Windows?
    // TODO: then check if binary exists
    Command::new("wl-copy").arg("--clear").status()?;
    // TODO: genericize over wl-copy, xclip, pbcopy, etc
    //   ref: https://github.com/atotto/clipboard/blob/e9e854e353882a018e9dc587e3757a8822958941/clipboard_unix.go
    Command::new("wl-copy")
        .arg("--trim-newline")
        .stdin(Stdio::piped())
        .spawn()?
        .stdin
        .ok_or_else(|| err_msg("stdin wasn't captured"))?
        .write_all(contents.as_bytes())?;

    Ok(())
}
