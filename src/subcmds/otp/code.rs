use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use termion::color;
use termion::style;

use crate::clipboard;
use crate::consts::{PASSWORD_STORE_CLIP_TIME, STORE_LEN};
use crate::otp::TOTPBuilder;
use crate::ui;
use crate::ui::UiResult;
use crate::util;
use crate::PassrsError;

use super::validate;

pub(crate) fn code(secret_name: String, clip: bool) -> Result<()> {
    let file = ui::display_matches_for_target(&secret_name)?;

    if let UiResult::Success(file) = file {
        let lines = util::decrypt_file_into_strings(&file)?;
        let file = file[*STORE_LEN..file.len() - 4].to_owned();
        let mut ret = Err(PassrsError::NoUriFound(file.clone()).into());

        for otp in lines {
            if validate::validate(&otp).is_ok() {
                ret = Ok(());

                let code = self::generate_totp(&otp)?;

                if clip {
                    clipboard::clip(&code, false)?;
                    println!(
                        "Copied token for {yellow}{}{reset} to the clipboard, which will clear in {} seconds.",
                        &file,
                        *PASSWORD_STORE_CLIP_TIME,
                        yellow = color::Fg(color::Yellow),
                        reset = style::Reset,
                    );
                } else {
                    let period = validate::get_period(&otp)?;

                    self::display_code(&code, period)?;
                }

                break;
            }
        }

        ret
    } else {
        Err(PassrsError::NoMatchesFound(secret_name).into())
    }
}

pub(crate) fn generate_totp<S>(otp: S) -> Result<String>
where
    S: AsRef<str>,
{
    let otp = otp.as_ref();
    let secret = validate::get_base32_secret(&otp)?;
    let period = validate::get_period(&otp)?;
    let algorithm = validate::get_algorithm(&otp)?;
    let digits = validate::get_digits(&otp)?;
    let auth = TOTPBuilder::default()
        .base32_secret(&secret)
        .period(period)
        .algorithm(algorithm)
        .output_len(digits)
        .build();

    let code = auth.generate();

    Ok(code)
}

pub(crate) fn display_code<S>(code: S, period: u64) -> Result<()>
where
    S: AsRef<str>,
{
    let code = code.as_ref();
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let duration = period - (time % period);
    let elapsed = (period - duration) as usize;
    let remaining = (duration % period) as usize;

    if elapsed == 0 {
        println!(
            "{} lasts {}s      \
             |{green}{bold}<{nobold}{:=<29}{reset}|",
            code,
            duration,
            "",
            green = color::Fg(color::Green),
            bold = style::Bold,
            nobold = style::NoBold,
            reset = style::Reset
        )
    } else {
        println!(
            "{} lasts {}s      \
             |{red}{:-<elapsed$}{green}{bold}<{reset}{green}{:=<remaining$}{reset}|",
            code,
            duration,
            "-",
            "",
            red = color::Fg(color::Red),
            green = color::Fg(color::Green),
            bold = style::Bold,
            reset = style::Reset,
            elapsed = elapsed + 1,
            remaining = remaining - 1
        );
    }

    Ok(())
}
