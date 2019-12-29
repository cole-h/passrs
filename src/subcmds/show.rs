use std::process::Command;

use failure::{err_msg, Fallible};
use ring::digest;

use crate::clipboard;
use crate::consts::PASSWORD_STORE_CLIP_TIME;
use crate::ui::{self, UiResult};
use crate::util;

pub fn show(clip: Option<Option<usize>>, pass_name: String) -> Fallible<()> {
    let file = ui::display_matches(&pass_name)?;

    match file {
        UiResult::Success(file) => {
            let password = util::decrypt_file_into_vec(file)?;

            if let Some(clip) = clip {
                let contents = match clip {
                    Some(line) => password
                        .get(line.saturating_sub(1))
                        .ok_or_else(|| err_msg(format!("File at line {} was empty", line)))?,
                    None => password.first().ok_or_else(|| err_msg("Vec was empty"))?,
                };
                let hash = hex::encode(digest::digest(&digest::SHA256, contents.as_bytes()));

                clipboard::clip(contents)?;

                // otherwise, the process is killed before it can spawn the unclip daemon
                std::thread::sleep(std::time::Duration::from_millis(50));

                // TODO: maybe abstract away command spawning? No easy way to do this,
                // though
                Command::new(std::env::current_exe()?)
                    .args(vec!["unclip", &PASSWORD_STORE_CLIP_TIME])
                    .env("PASSRS_UNCLIP_HASH", hash)
                    .spawn()?;
            } else {
                for line in password {
                    println!("{}", line);
                }
            }
        }
        UiResult::CopiedToClipboard(file) => {
            println!("{}", &file[..file.len() - 4]);
        }
        _ => {}
    }

    Ok(())
}
