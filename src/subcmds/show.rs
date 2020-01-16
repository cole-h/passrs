use anyhow::{Context, Result};
use termion::color;
use termion::style;

use crate::clipboard;
use crate::consts::{PASSWORD_STORE_CLIP_TIME, STORE_LEN};
use crate::ui;
use crate::ui::UiResult;
use crate::util;

#[allow(clippy::option_option)]
pub fn show(secret_name: String, clip: Option<Option<usize>>) -> Result<()> {
    let file = ui::display_matches_for_target(&secret_name)?;

    match file {
        UiResult::Success(file) => {
            let password = util::decrypt_file_into_strings(&file)?;

            match clip {
                Some(clip) => {
                    let contents = match clip {
                        Some(line) => password
                            .get(line.saturating_sub(1))
                            .with_context(|| format!("File at line {} was empty", line))?,
                        None => password.first().with_context(|| "Vec was empty")?,
                    };

                    clipboard::clip(contents, false)?;
                    println!(
                        "Copied {yellow}{}{reset} to the clipboard, which will clear in {} seconds.",
                        &file[*STORE_LEN..file.len() - 4],
                        *PASSWORD_STORE_CLIP_TIME,
                        yellow = color::Fg(color::Yellow),
                        reset = style::Reset,
                    );
                }
                _ => {
                    println!(
                        "Contents of {yellow}{}{reset}",
                        &file[*STORE_LEN..file.len() - 4],
                        yellow = color::Fg(color::Yellow),
                        reset = style::Reset
                    );
                    for line in &password {
                        println!("{}", line,);
                    }
                }
            }
        }
        UiResult::CopiedToClipboard(file) => {
            println!(
                "Copied {yellow}{}{reset} to the clipboard, which will clear in {} seconds.",
                &file[*STORE_LEN..file.len() - 4],
                *PASSWORD_STORE_CLIP_TIME,
                yellow = color::Fg(color::Yellow),
                reset = style::Reset,
            );
        }
        UiResult::SpawnEditor(file) => {
            crate::subcmds::edit::edit(file)?;
        }
        _ => {}
    }

    Ok(())
}
