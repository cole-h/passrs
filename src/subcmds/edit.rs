use failure::Fallible;

use std::process::{Command, Stdio};

use crate::consts::EDITOR;

pub fn edit(pass_name: String) -> Fallible<String> {
    // TODO: call to this should only be AFTER ui::display_matches
    // TODO: wrap command spawning
    // TODO: open file in a secure tmp file (/tmp/shm?)
    // let path = secure_file(full_path)
    Command::new(EDITOR.to_string())
        .arg(&pass_name)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    // overwrite file at original location with new contents
    // zeroize/drop tmp file

    Ok(format!("Edit secret for {} using {}", pass_name, *EDITOR))
}
