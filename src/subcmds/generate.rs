use std::io::{self, Write};
use std::process::Command;
use termion::input::TermRead;

use data_encoding::HEXLOWER;
use failure::Fallible;
use ring::digest;
use termion::{color, style};

use crate::clipboard;
use crate::consts::{
    PASSWORD_STORE_CHARACTER_SET, PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS,
    PASSWORD_STORE_CLIP_TIME, PASSWORD_STORE_GENERATED_LENGTH,
};
use crate::error::PassrsError;
use crate::util;

pub fn generate(
    no_symbols: bool,
    clip: bool,
    in_place: bool,
    force: bool,
    pass_name: String,
    pass_length: Option<usize>,
) -> Fallible<()> {
    let path = util::canonicalize_path(&pass_name)?;

    util::create_descending_dirs(&path)?;

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    if !force && util::path_exists(&path)? {
        write!(
            stdout,
            "An entry exists for {}. Overwrite it? [y/N] ",
            pass_name
        )?;
        io::stdout().flush()?;

        match stdin.read_line()? {
            Some(reply)
                if reply.chars().nth(0) == Some('y') || reply.chars().nth(0) == Some('Y') =>
            {
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

    let password_bytes = util::generate_chars_from_set(set, len)?;
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
        let hash = HEXLOWER.encode(digest::digest(&digest::SHA256, &password_bytes).as_ref());
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
        let files = util::find_target_single(&pass_name)?;
        assert_eq!(files.len(), 1);
        let mut existing = util::decrypt_file_into_strings(files[0].clone())?;
        existing[0] = password;

        let existing = existing.join("\n");
        let existing = existing.as_bytes();
        util::encrypt_bytes_into_file(existing, &path)?;
        util::commit(format!("Replace generated secret for {}", pass_name))?;
        Ok(())
    } else {
        util::encrypt_bytes_into_file(&password_bytes, &path)?;
        util::commit(format!("Save generated secret for {}", pass_name))?;
        Ok(())
    }
}
