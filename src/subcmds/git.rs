use std::process::{Command, Stdio};

use failure::Fallible;

use crate::consts::{PASSRS_GIT_BINARY, PASSWORD_STORE_DIR};

pub fn git(args: Vec<String>) -> Fallible<()> {
    // TODO: generalize command spawning
    Command::new(&*PASSRS_GIT_BINARY)
        .args(&args)
        .current_dir(PASSWORD_STORE_DIR.to_owned())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    Ok(())
}
