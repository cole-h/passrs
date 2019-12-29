use std::process::Command;

use failure::Fallible;
use rand::Rng;
use ring::digest;
use termion::color;
use termion::style;

use crate::clipboard;
use crate::consts::{
    PASSWORD_STORE_CHARACTER_SET, PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS,
    PASSWORD_STORE_CLIP_TIME, PASSWORD_STORE_DIR, PASSWORD_STORE_GENERATED_LENGTH,
};
use crate::error::PassrsError;
use crate::util;

pub fn generate(
    rng: &mut impl Rng,
    no_symbols: bool,
    clip: bool,
    in_place: bool,
    force: bool,
    pass_name: String,
    pass_length: Option<usize>,
) -> Fallible<String> {
    let path = format!("{}/{}.gpg", *PASSWORD_STORE_DIR, pass_name);

    if util::path_exists(&path).is_err() && !force && !in_place {
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

    // NOTE: default sets defined in consts.rs
    let set = if no_symbols {
        &*PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS
    } else {
        &*PASSWORD_STORE_CHARACTER_SET
    };
    let len = if let Some(length) = pass_length {
        length
    } else {
        *PASSWORD_STORE_GENERATED_LENGTH
    };
    let mut password_bytes: Vec<u8> = Vec::with_capacity(len as usize);

    for _ in 0..len {
        let idx = rng.gen_range(0, set.len());
        password_bytes.push(set[idx]);
    }

    let password = std::str::from_utf8(&password_bytes)?.to_owned();
    println!(
        "{bold}The generated password for {underline}{}{reset}{bold} is:\n{yellow}{bold}{}{reset}",
        pass_name,
        password,
        underline = style::Underline,
        bold = style::Bold,
        yellow = color::Fg(color::Yellow),
        reset = style::Reset,
    );

    if clip {
        let hash = hex::encode(digest::digest(&digest::SHA256, &password_bytes));
        let args = vec![
            "unclip",
            if force { "--force" } else { "" },
            &PASSWORD_STORE_CLIP_TIME,
        ];
        let args = args.iter().filter(|&&x| x != "").collect::<Vec<_>>();

        clipboard::clip(&password)?;

        // otherwise, the process doesn't live long enough
        std::thread::sleep(std::time::Duration::from_millis(50));

        Command::new(std::env::current_exe()?)
            .args(args)
            .env("PASSRS_UNCLIP_HASH", hash)
            .spawn()?;
    }

    if in_place {
        // TODO: ensure file[0] is actually the proper entry
        let files = util::search_entries(&pass_name)?;
        assert_eq!(files.len(), 1);
        let mut existing = util::decrypt_file_into_vec(files[0].clone())?;
        existing[0] = password;

        let existing = existing.join("\n");
        let existing = existing.as_bytes();
        util::encrypt_bytes_into_file(&path, existing)?;
        Ok(format!("Replace generated secret for {}", pass_name))
    } else {
        util::encrypt_bytes_into_file(&path, &password_bytes)?;
        Ok(format!("Save generated secret for {}", pass_name))
    }
}
