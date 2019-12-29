use failure::Fallible;

use crate::consts::PASSWORD_STORE_DIR;
use crate::error::PassrsError;
use crate::util;

pub fn insert(echo: bool, multiline: bool, force: bool, pass_name: String) -> Fallible<String> {
    let path = format!("{}/{}.gpg", *PASSWORD_STORE_DIR, pass_name);

    if util::path_exists(&path).is_err() && !force {
        match rprompt::prompt_reply_stdout(&format!(
            "An entry exists for {}. Overwrite it? [y/N] ",
            pass_name
        )) {
            Ok(reply) if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') => {
                std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&path)?;
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    let password = if echo {
        Some(rprompt::prompt_reply_stdout(&format!(
            "Enter password for {}: ",
            pass_name
        ))?)
    } else if multiline {
        println!(
            "Enter the contents of {} and press Ctrl+D when finished:\n",
            pass_name
        );
        let mut input = Vec::new();

        use std::io::BufRead;
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            input.push(line?);
        }

        Some(input.join("\n"))
    } else {
        // TODO: send PR upstream to revert
        // d4cc03171a3bde2a701b261e0f3ec227a43a3b51 "Permit lack of newline,
        // given possible piping or EOF. (#29)" OR implement an EOF-reader
        let input =
            rpassword::prompt_password_stdout(&format!("Enter password for {}: ", pass_name))?;
        let check =
            rpassword::prompt_password_stdout(&format!("Retype password for {}: ", pass_name))?;

        if input == check {
            Some(input)
        } else {
            // TODO: zeroize or drop input and check
            return Err(PassrsError::PasswordsDontMatch.into());
        }
    };

    // if we prompted the user for a password and got one
    if let Some(password) = password {
        util::encrypt_bytes_into_file(path, password.as_bytes())?;
    }

    Ok(format!("Add given password for {} to store", pass_name))
}
