use anyhow::Result;

use crate::subcmds::otp::validate;
use crate::util;

pub fn append(echo: bool, secret_name: String, from_secret: bool) -> Result<()> {
    let path = util::canonicalize_path(&secret_name)?;

    if from_secret {
        let secret = util::prompt_for_secret(echo, &secret_name)?;

        // if we prompted the user for a secret and got one
        if let Some(secret) = secret {
            let secret = format!("otpauth://totp/{}?secret={}", secret_name, secret);
            validate::validate(&secret)?;
            util::append_encrypted_bytes(secret.as_bytes(), path)?;
        }
    } else {
        let secret = util::prompt_for_secret(echo, &secret_name)?;

        if let Some(secret) = secret {
            validate::validate(&secret)?;
            util::append_encrypted_bytes(secret.as_bytes(), path)?;
        }
    }

    util::commit(format!("Append OTP secret for {}", secret_name))?;
    Ok(())
}
