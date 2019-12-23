// use crate::Result;
use std::process::{Command, ExitStatus, Stdio};

pub(crate) mod cp;
pub(crate) mod edit;
pub(crate) mod find;
pub(crate) mod generate;
pub(crate) mod git;
pub(crate) mod grep;
pub(crate) mod init;
pub(crate) mod insert;
pub(crate) mod ls;
pub(crate) mod mv;
// pub(crate) mod otp;
pub(crate) mod rm;
pub(crate) mod show;
pub(crate) mod unclip;

pub(crate) mod otp;

pub(crate) fn _run_command(bin: &str, args: Vec<&str>) -> ExitStatus {
    Command::new(bin)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| println!("Failed to execute command {}", err))
        .expect("failed to execute process")
}
