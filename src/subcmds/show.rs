use std::io::{self, Write};

use anyhow::{Context, Result};
use termion::color;
use termion::style;

use crate::clipboard;
use crate::consts::{PASSWORD_STORE_CLIP_TIME, STORE_LEN};
use crate::ui;
use crate::ui::UiResult;
use crate::util;

use super::edit;

#[allow(clippy::option_option)]
pub(crate) fn show(secret_name: String, clip: Option<Option<usize>>) -> Result<()> {
    match ui::display_matches_for_target(&secret_name)? {
        UiResult::Success(file) => {
            let password = util::decrypt_file_into_strings(&file)?;

            match clip {
                Some(clip) => {
                    let file = &file[*STORE_LEN..file.rfind(".gpg").unwrap_or_else(|| file.len())];
                    let contents = match clip {
                        Some(line) => password
                            .get(line.saturating_sub(1))
                            .with_context(|| format!("File at line {} was empty", line))?,
                        None => password.first().with_context(|| "Vec was empty")?,
                    };

                    clipboard::clip(contents, false)?;
                    writeln!(
                        io::stdout(),
                        "Copied {yellow}{}{reset} to the clipboard, which will clear in {} seconds.",
                        file,
                        *PASSWORD_STORE_CLIP_TIME,
                        yellow = color::Fg(color::Yellow),
                        reset = style::Reset,
                    )?;
                }
                _ => {
                    if termion::is_tty(&io::stdout()) {
                        let file =
                            &file[*STORE_LEN..file.rfind(".gpg").unwrap_or_else(|| file.len())];

                        writeln!(
                            io::stdout(),
                            "Contents of {yellow}{}{reset}",
                            file,
                            yellow = color::Fg(color::Yellow),
                            reset = style::Reset
                        )?;

                        for line in &password {
                            writeln!(io::stdout(), "{}", line)?;
                        }
                    } else {
                        // if stdout is not a tty, it is being piped, so only
                        // print the first line
                        write!(io::stdout(), "{}", &password[0])?;
                    }
                }
            }
        }
        UiResult::CopiedToClipboard(file) => {
            let file = &file[*STORE_LEN..file.rfind(".gpg").unwrap_or_else(|| file.len())];

            writeln!(
                io::stdout(),
                "Copied {yellow}{}{reset} to the clipboard, which will clear in {} seconds.",
                file,
                *PASSWORD_STORE_CLIP_TIME,
                yellow = color::Fg(color::Yellow),
                reset = style::Reset,
            )?;
        }
        UiResult::SpawnEditor(file) => {
            let file = &file[..file.rfind(".gpg").unwrap_or_else(|| file.len())];

            edit::edit(&file)?;
        }
        _ => {}
    }

    Ok(())
}
