use std::process::Command;

use anyhow::Context;
use data_encoding::HEXLOWER;
use ring::digest;
use termion::{color, style};

use crate::clipboard;
use crate::consts::{PASSWORD_STORE_CLIP_TIME, PASSWORD_STORE_DIR};
use crate::ui::{self, UiResult};
use crate::util;
use crate::Result;

pub fn show(clip: Option<Option<usize>>, pass_name: String) -> Result<()> {
    let file = ui::display_matches_for_target(&pass_name)?;

    match file {
        UiResult::Success(file) => {
            let password = util::decrypt_file_into_strings(file)?;

            if let Some(clip) = clip {
                let contents = match clip {
                    Some(line) => password
                        .get(line.saturating_sub(1))
                        .with_context(|| format!("File at line {} was empty", line))?,
                    None => password.first().with_context(|| "Vec was empty")?,
                };
                let hash =
                    HEXLOWER.encode(digest::digest(&digest::SHA256, contents.as_bytes()).as_ref());

                clipboard::clip(contents)?;

                // otherwise, the unclip daemon doesn't have a chance to spawn
                std::thread::sleep(std::time::Duration::from_millis(50));

                // TODO: maybe abstract away command spawning? No easy way to do this,
                // though
                Command::new(std::env::current_exe()?)
                    .args(vec!["unclip", &PASSWORD_STORE_CLIP_TIME])
                    .env("PASSRS_UNCLIP_HASH", hash)
                    .spawn()?;
            } else {
                println!("{}", pass_name);
                for line in password {
                    println!(
                        "{yellow}{}{reset}",
                        line,
                        yellow = color::Fg(color::Yellow),
                        reset = style::Reset,
                    );
                }
            }
        }
        UiResult::CopiedToClipboard(file) => {
            println!(
                "Copied {yellow}{}{reset} to the clipboard, which will clear in {} seconds.",
                &file[PASSWORD_STORE_DIR.len()..file.len() - 4],
                *PASSWORD_STORE_CLIP_TIME,
                yellow = color::Fg(color::Yellow),
                reset = style::Reset,
            );
        }
        _ => {}
    }

    Ok(())
}
