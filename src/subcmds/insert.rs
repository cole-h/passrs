use crate::consts::{DEFAULT_STORE_PATH, EDITOR};
use crate::utils;

pub fn insert(echo: bool, multiline: bool, force: bool, pass_name: String) -> Option<String> {
    // if multiline, spawn editor
    let commit_message = if multiline {
        format!("Save secret to {} using {:?}", pass_name, *EDITOR)
    } else {
        format!("Save secret to {}", pass_name)
    };

    // if path/file exists && !force {
    //   warn: path exists
    //   return
    // }

    if utils::verify_path(&format!("{}/{}", *DEFAULT_STORE_PATH, pass_name)).is_ok() && !force {
        // warn: path exists
        // return
    }

    let password = if echo {
        rprompt::prompt_reply_stdout(&format!("Enter password for {}", pass_name)).ok()
    } else if multiline {
        None
    } else {
        let input = rpassword::prompt_password_stdout(&format!("Enter password for {}", pass_name))
            .unwrap_or(String::from("1"));
        let check =
            rpassword::prompt_password_stdout(&format!("Retype password for {}", pass_name))
                .unwrap_or(String::from("2"));

        if input == check {
            Some(input)
        } else {
            // TODO: zeroize or drop input and check
            eprintln!("Error: the entered passwords do not match.");
            return None;
        }
    };

    // if we prompted the user for a password and got one
    if let Some(password) = password {
        let _ = password;
        // save_password(pass_name, password)
    }

    Some(commit_message)
}
