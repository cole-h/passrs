use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::str;

use termion::color;
use termion::style;

use crate::clipboard;
use crate::consts::{
    PASSWORD_STORE_CHARACTER_SET, PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS,
    PASSWORD_STORE_CLIP_TIME, PASSWORD_STORE_GENERATED_LENGTH, PASSWORD_STORE_UMASK,
};
use crate::util;
use crate::util::EditMode;
use crate::{Flags, PassrsError, Result};

pub(crate) fn generate(secret_name: String, length: Option<usize>, flags: Flags) -> Result<()> {
    let clip = flags.clip;
    let force = flags.force;
    let in_place = flags.in_place;
    let no_symbols = flags.no_symbols;
    let path = util::canonicalize_path(&secret_name)?;

    util::create_dirs_to_file(&path)?;

    if !force && !in_place && util::path_exists(&path)? {
        let prompt = format!("An entry exists for {}. Overwrite it?", secret_name);

        if util::prompt_yesno(prompt)? {
            fs::OpenOptions::new()
                .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
                .write(true)
                .truncate(!in_place)
                .open(&path)?;
        } else {
            return Err(PassrsError::UserAbort.into());
        }
    }

    // NOTE: default character sets defined in consts.rs
    let set = if no_symbols {
        &*PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS
    } else {
        &*PASSWORD_STORE_CHARACTER_SET
    };
    let len = if let Some(length) = length {
        length
    } else {
        *PASSWORD_STORE_GENERATED_LENGTH
    };

    let secret_bytes = util::generate_chars_from_set(set, len)?;
    let secret = str::from_utf8(&secret_bytes)?.to_owned();

    if clip {
        clipboard::clip(&secret, force)?;

        writeln!(
            io::stdout(),
            "Copied {yellow}{}{reset} to the clipboard, which will clear in {} seconds.",
            &secret_name,
            *PASSWORD_STORE_CLIP_TIME,
            yellow = color::Fg(color::Yellow),
            reset = style::Reset,
        )?;
    }

    if in_place {
        let mut existing = util::decrypt_file_into_strings(&path)?;
        existing[0] = secret.clone();

        let existing = existing.join("\n");
        let existing = existing.as_bytes();

        util::encrypt_bytes_into_file(existing, &path, EditMode::Clobber)?;
        util::commit(
            Some([&path]),
            format!("Replace generated secret for {}", secret_name),
        )?;

        if !clip {
            writeln!(io::stdout(),
                "{bold}The generated secret for {underline}{}{reset}{bold} is:\n{yellow}{bold}{}{reset}",
                secret_name,
                secret,
                bold = style::Bold,
                underline = style::Underline,
                reset = style::Reset,
                yellow = color::Fg(color::Yellow),
            )?;
        }
    } else {
        util::encrypt_bytes_into_file(&secret_bytes, &path, EditMode::Clobber)?;
        util::commit(
            Some([&path]),
            format!("Save generated secret for {}", secret_name),
        )?;

        if !clip {
            writeln!(io::stdout(),
                "{bold}The generated secret for {underline}{}{reset}{bold} is:\n{yellow}{bold}{}{reset}",
                secret_name,
                secret,
                bold = style::Bold,
                underline = style::Underline,
                reset = style::Reset,
                yellow = color::Fg(color::Yellow),
            )?;
        }
    }

    Ok(())
}
