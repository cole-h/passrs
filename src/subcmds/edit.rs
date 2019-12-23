use std::process::{Command, Stdio};

use crate::consts::EDITOR;

pub fn edit(pass_name: String) -> Option<String> {
    // TODO: fuzzy find pass_name so it doesn't require absolute paths
    // let full_path = fuzzy_find(default_path, pass_name) -> Entire path;
    // TODO: wrap command spawning
    // TODO: open file in a secure tmp file (/tmp/shm?)
    // let path = secure_file(full_path)
    Command::new(EDITOR.to_string())
        .arg(&pass_name)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    // overwrite file at original location with new contents
    // zeroize/drop tmp file

    Some(format!("Edit secret for {} using {}", pass_name, *EDITOR))
}
