use std::fs;
use std::os::unix::fs::OpenOptionsExt;

use anyhow::Result;

use crate::consts::PASSWORD_STORE_UMASK;
use crate::util;
use crate::util::EditMode;
use crate::Flags;
use crate::PassrsError;

use super::code;
use super::validate;

pub fn insert(
    secret_name: String,
    algo: Option<String>,
    digits: Option<usize>,
    period: Option<u32>,
    flags: Flags,
) -> Result<()> {
    let force = flags.force;
    let echo = flags.echo;
    let generate = flags.generate;
    let from_secret = flags.from_secret;
    let path = util::canonicalize_path(&secret_name)?;

    if !force && util::path_exists(&path)? {
        let prompt = format!("An entry exists for {}. Overwrite it?", secret_name);

        if util::prompt_yesno(prompt)? {
            fs::OpenOptions::new()
                .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
                .write(true)
                .truncate(true)
                .open(&path)?;
        } else {
            return Err(PassrsError::UserAbort.into());
        }
    }

    if from_secret {
        let secret = util::prompt_for_secret(echo, false, &secret_name)?;

        if let Some(secret) = secret {
            let mut secret = format!("otpauth://totp/{}?secret={}", secret_name, secret);

            if let Some(algo) = algo {
                let algo = algo.to_ascii_lowercase();
                match algo.as_ref() {
                    "sha512" => secret += "&algorithm=SHA512",
                    "sha256" => secret += "&algorithm=SHA256",
                    "sha1" => secret += "&algorithm=SHA1",
                    _ => return Err(PassrsError::InvalidHashAlgorithm(algo).into()),
                }
            }
            if let Some(period) = period {
                secret += &format!("&period={}", period);
            }
            if let Some(digits) = digits {
                secret += &format!("&digits={}", digits);
            }

            validate::validate(&secret)?;

            if generate {
                let code = code::generate_totp(&secret)?;
                let period = validate::get_period(&secret)?;
                code::display_code(&code, period)?;
            }

            util::encrypt_bytes_into_file(secret.as_bytes(), path, EditMode::Clobber)?;
            util::commit(format!("Add OTP secret for {} to store", secret_name))?;
        }
    } else {
        let secret = util::prompt_for_secret(echo, false, &secret_name)?;

        if let Some(secret) = secret {
            validate::validate(&secret)?;

            if generate {
                let code = code::generate_totp(&secret)?;
                let period = validate::get_period(&secret)?;
                code::display_code(&code, period)?;
            }

            util::encrypt_bytes_into_file(secret.as_bytes(), path, EditMode::Clobber)?;
            util::commit(format!("Add OTP secret for {} to store", secret_name))?;
        }
    }

    Ok(())
}
