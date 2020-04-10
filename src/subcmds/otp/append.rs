use std::path::PathBuf;

use anyhow::Result;

use crate::util::{self, EditMode};
use crate::Flags;

use super::{code, validate};

pub(crate) fn append(
    secret_name: String,
    algo: Option<String>,
    digits: Option<usize>,
    period: Option<u32>,
    flags: Flags,
) -> Result<()> {
    let echo = flags.echo;
    let generate = flags.generate;
    let from_secret = flags.from_secret;
    let path = util::canonicalize_path(&secret_name)?;

    if from_secret {
        let secret = util::prompt_for_secret(&secret_name, echo, false)?;

        if let Some(secret) = secret {
            let mut secret = format!("otpauth://totp/{}?secret={}", secret_name, secret);

            if let Some(algo) = algo {
                match algo.to_ascii_lowercase().as_ref() {
                    "sha512" => secret += "&algorithm=SHA512",
                    "sha256" => secret += "&algorithm=SHA256",
                    _ => secret += "&algorithm=SHA1",
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
                code::display_code(&secret)?;
            }

            util::encrypt_bytes_into_file(secret.as_bytes(), path, EditMode::Append)?;
            util::commit(
                None::<&[PathBuf]>,
                format!("Append OTP secret for {}", secret_name),
            )?;
        }
    } else {
        let secret = util::prompt_for_secret(&secret_name, echo, false)?;

        if let Some(secret) = secret {
            validate::validate(&secret)?;

            if generate {
                code::display_code(&secret)?;
            }

            util::encrypt_bytes_into_file(secret.as_bytes(), path, EditMode::Append)?;
            util::commit(
                None::<&[PathBuf]>,
                format!("Append OTP secret for {}", secret_name),
            )?;
        }
    }

    Ok(())
}
