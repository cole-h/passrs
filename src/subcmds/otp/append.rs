use anyhow::Result;

use crate::subcmds::otp::validate;
use crate::util;
use crate::util::FileMode;

pub fn append(echo: bool, secret_name: String, from_secret: bool) -> Result<()> {
    let path = util::canonicalize_path(&secret_name)?;

    if from_secret {
        let secret = util::prompt_for_secret(echo, &secret_name)?;

        // if we prompted the user for a secret and got one
        if let Some(secret) = secret {
            let secret = format!("otpauth://totp/{}?secret={}", secret_name, secret);
            validate::validate(&secret)?;
            util::encrypt_bytes_into_file(secret.as_bytes(), path, FileMode::Append)?;
        }
    } else {
        let secret = util::prompt_for_secret(echo, &secret_name)?;

        if let Some(secret) = secret {
            validate::validate(&secret)?;
            util::encrypt_bytes_into_file(secret.as_bytes(), path, FileMode::Append)?;
        }
    }

    util::commit(format!("Append OTP secret for {}", secret_name))?;
    Ok(())
}
