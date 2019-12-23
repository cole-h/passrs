use std::process::{Command, Stdio};

use crate::consts::PASSWORD_STORE_DIR;

pub fn git(args: Vec<String>) {
    let git_binary = envmnt::get_or("PASSRS_GIT_BINARY", "/usr/bin/git");

    Command::new(&git_binary)
        .args(&args)
        .current_dir(PASSWORD_STORE_DIR.to_string())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|err| {
            println!(
                "Failed to execute command: \"{} {:?}\": {}",
                git_binary, args, err
            )
        })
        .ok();
}
