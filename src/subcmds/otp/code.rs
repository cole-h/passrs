use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use termion::{color, style};

use crate::consts::{PASSWORD_STORE_CLIP_TIME, STORE_LEN};
use crate::otp::TOTPBuilder;
use crate::ui::{self, UiResult};
use crate::{clipboard, util};
use crate::{PassrsError, Result};

use super::validate;

pub(crate) fn code(secret_name: String, clip: bool) -> Result<()> {
    let file = ui::display_matches_for_target(&secret_name)?;

    match file {
        UiResult::Success(file) => {
            let lines = util::decrypt_file_into_strings(&file)?;
            let file = file[*STORE_LEN..file.rfind(".gpg").unwrap()].to_owned();

            for otp in lines {
                if validate::validate(&otp).is_ok() {
                    if clip {
                        let code = self::generate_totp(&otp)?;

                        clipboard::clip(&code, false)?;
                        writeln!(io::stdout(),
                                 "Copied token for {yellow}{}{reset} to the clipboard, which will clear in {} seconds.",
                                 &file,
                                 *PASSWORD_STORE_CLIP_TIME,
                                 yellow = color::Fg(color::Yellow),
                                 reset = style::Reset,
                        )?;
                    } else {
                        self::display_code(&otp)?;
                    }

                    return Ok(());
                }
            }

            Err(PassrsError::NoUriFound(file).into())
        }
        _ => Err(PassrsError::NoMatchesFound(secret_name).into()),
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

pub(crate) fn display_code<S>(otp: S) -> Result<()>
where
    S: AsRef<str>,
{
    let otp = otp.as_ref();
    let code = self::generate_totp(&otp)?;
    let period = validate::get_period(&otp)?;
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let duration = period - (time % period);
    let elapsed = (period - duration) as usize;
    let remaining = (duration % period) as usize;

    if termion::is_tty(&io::stdout()) {
        if elapsed == 0 {
            writeln!(
                io::stdout(),
                "{} lasts {}s      \
                 |{green}{bold}<{nobold}{:=<29}{reset}|",
                code,
                duration,
                "",
                green = color::Fg(color::Green),
                bold = style::Bold,
                nobold = style::NoBold,
                reset = style::Reset
            )?
        } else {
            writeln!(
                io::stdout(),
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
            )?;
        }
    } else {
        write!(io::stdout(), "{}", code)?;
    }

    Ok(())
}
