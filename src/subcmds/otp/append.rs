use anyhow::Result;

use crate::subcmds::otp::validate;
use crate::util;
use crate::util::FileMode;

pub fn append(
    echo: bool,
    secret_name: String,
    from_secret: bool,
    algo: Option<String>,
    period: Option<u32>,
    digits: Option<usize>,
) -> Result<()> {
    let path = util::canonicalize_path(&secret_name)?;

    if from_secret {
        let secret = util::prompt_for_secret(echo, false, &secret_name)?;

        // if we prompted the user for a secret and got one
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
            util::encrypt_bytes_into_file(secret.as_bytes(), path, FileMode::Append)?;
            util::commit(format!("Append OTP secret for {}", secret_name))?;
        }
    } else {
        let secret = util::prompt_for_secret(echo, false, &secret_name)?;

        if let Some(secret) = secret {
            validate::validate(&secret)?;
            util::encrypt_bytes_into_file(secret.as_bytes(), path, FileMode::Append)?;
            util::commit(format!("Append OTP secret for {}", secret_name))?;
        }
    }

    Ok(())
}
