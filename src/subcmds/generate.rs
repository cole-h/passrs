use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::process::Command;
use std::str;
use std::thread;
use std::time;

use anyhow::Result;
use data_encoding::HEXLOWER;
use ring::digest;
use termion::color;
use termion::input::TermRead;
use termion::style;

use crate::clipboard;
use crate::consts::{
    PASSWORD_STORE_CHARACTER_SET, PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS,
    PASSWORD_STORE_CLIP_TIME, PASSWORD_STORE_GENERATED_LENGTH, PASSWORD_STORE_UMASK,
};
use crate::util;
use crate::util::FileMode;
use crate::PassrsError;

pub fn generate(
    no_symbols: bool,
    clip: bool,
    in_place: bool,
    force: bool,
    pass_name: String,
    pass_length: Option<usize>,
) -> Result<()> {
    let path = util::canonicalize_path(&pass_name)?;
    util::create_descending_dirs(&path)?;

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if !force && util::path_exists(&path)? {
        write!(
            stdout,
            "An entry exists for {}. Overwrite it? [y/N] ",
            pass_name
        )?;
        io::stdout().flush()?;

        match stdin.read_line()? {
            Some(reply) if reply.starts_with('y') || reply.starts_with('Y') => {
                fs::OpenOptions::new()
                    .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
                    .write(true)
                    .truncate(!in_place)
                    .open(&path)?;
            }
            _ => return Err(PassrsError::UserAbort.into()),
        }
    }

    // NOTE: default character sets defined in consts.rs
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
    let password = str::from_utf8(&password_bytes)?.to_owned();

    println!(
        "{bold}The generated password for {underline}{}{reset}{bold} is:\n{yellow}{bold}{}{reset}",
        pass_name,
        password,
        bold = style::Bold,
        underline = style::Underline,
        reset = style::Reset,
        yellow = color::Fg(color::Yellow),
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
        // Otherwise, the process doesn't live long enough to spawn the unclip
        // daemon
        thread::sleep(time::Duration::from_millis(50));
        Command::new(env::current_exe()?)
            .args(args)
            .env("PASSRS_UNCLIP_HASH", hash)
            .spawn()?;
    }

    if in_place {
        let mut existing = util::decrypt_file_into_strings(&path)?;
        existing[0] = password;

        let existing = existing.join("\n");
        let existing = existing.as_bytes();

        util::encrypt_bytes_into_file(existing, &path, FileMode::Clobber)?;
        util::commit(format!("Replace generated secret for {}", pass_name))?;
    } else {
        util::encrypt_bytes_into_file(&password_bytes, &path, FileMode::Clobber)?;
        util::commit(format!("Save generated secret for {}", pass_name))?;
    }

    Ok(())
}
