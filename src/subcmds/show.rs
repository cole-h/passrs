use std::env;
use std::process::Command;
use std::thread;
use std::time;

use anyhow::{Context, Result};
use data_encoding::HEXLOWER;
use ring::digest;
use termion::color;
use termion::style;
use zeroize::Zeroize;

use crate::clipboard;
use crate::consts::{PASSWORD_STORE_CLIP_TIME, PASSWORD_STORE_DIR};
use crate::ui;
use crate::ui::UiResult;
use crate::util;

#[allow(clippy::option_option)]
pub fn show(clip: Option<Option<usize>>, pass_name: String) -> Result<()> {
    let file = ui::display_matches_for_target(&pass_name)?;

    match file {
        UiResult::Success(file) => {
            let mut password = util::decrypt_file_into_strings(&file)?;

            match clip {
                Some(clip) => {
                    let contents = match clip {
                        Some(line) => password
                            .get(line.saturating_sub(1))
                            .with_context(|| format!("File at line {} was empty", line))?,
                        None => password.first().with_context(|| "Vec was empty")?,
                    };
                    let hash = HEXLOWER
                        .encode(digest::digest(&digest::SHA256, contents.as_bytes()).as_ref());

                    clipboard::clip(contents)?;

                    // otherwise, the unclip daemon doesn't have a chance to spawn
                    thread::sleep(time::Duration::from_millis(50));

                    // TODO: maybe abstract away command spawning? No easy way to do this,
                    // though
                    Command::new(env::current_exe()?)
                        .args(vec!["unclip", &PASSWORD_STORE_CLIP_TIME])
                        .env("PASSRS_UNCLIP_HASH", hash)
                        .spawn()?;
                }
                _ => {
                    println!("{}", &file[PASSWORD_STORE_DIR.len()..file.len() - 4]);
                    for line in &password {
                        println!(
                            "{yellow}{}{reset}",
                            line,
                            yellow = color::Fg(color::Yellow),
                            reset = style::Reset,
                        );
                    }
                }
            }

            password.zeroize();
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
        UiResult::SpawnEditor(file) => {
            crate::subcmds::edit::edit(file)?;
        }
        _ => {}
    }

    Ok(())
}
