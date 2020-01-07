use anyhow::{Context, Result};

use crate::clipboard;
use crate::otp::{HashAlgorithm, TOTPBuilder};
use crate::subcmds::otp::validate;
use crate::ui;
use crate::ui::UiResult;
use crate::util;

pub fn code(clip: bool, pass_name: String) -> Result<()> {
    let file = ui::display_matches_for_target(&pass_name)?;

    if let UiResult::Success(file) = file {
        let otp = util::decrypt_file_into_strings(file)?;
        let otp = otp.first().with_context(|| "Vec was empty")?;

        // Ensure `otp` is a valid URI
        validate::validate(otp)?;

        let base32_secret = validate::get_base32_secret(otp)?;
        let period = validate::get_period(otp)?;
        let algorithm = validate::get_algorithm(otp)?;
        let digits = validate::get_digits(otp)?;

        let code = generate_totp(base32_secret, period, algorithm, digits)?;

        if clip {
            clipboard::clip(&code, false)?;
        } else {
            println!("{}", code);
        }
    }

    Ok(())
}

fn generate_totp<S>(
    secret: S,
    period: u64,
    algorithm: HashAlgorithm,
    digits: usize,
) -> Result<String>
where
    S: Into<String>,
{
    let secret = secret.into();
    let auth = TOTPBuilder::new()
        .base32_secret(&secret)
        .period(period)
        .algorithm(algorithm)
        .output_length(digits)
        .build();
    let code = auth.generate();

    Ok(code)
}
