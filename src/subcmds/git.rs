use std::process::{Command, Stdio};

use crate::consts::{PASSRS_GIT_BINARY, PASSWORD_STORE_DIR};
use crate::Result;

pub(crate) fn git(args: Vec<String>) -> Result<()> {
    Command::new(&*PASSRS_GIT_BINARY)
        .args(&args)
        .current_dir(&*PASSWORD_STORE_DIR)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(())
}
