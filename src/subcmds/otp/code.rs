use failure::{err_msg, Fallible};

use crate::clipboard;
use crate::otp::{HOTPBuilder, HashAlgorithm, TOTPBuilder};
use crate::subcmds::otp::validate;
use crate::ui::{self, UiResult};
use crate::util;

pub fn code(clip: bool, pass_name: String) -> Fallible<()> {
    let file = ui::display_matches_for_target(&pass_name)?;

    if let UiResult::Success(file) = file {
        let otp = util::decrypt_file_into_strings(file)?;
        let otp = otp.first().ok_or_else(|| err_msg("Vec was empty"))?;

        // Ensure `otp` is a valid URI
        validate::validate(otp)?;

        // let otp_type = validate::get_type(otp)?;
        // let counter = validate::get_counter(otp)?;
        let base32_secret = validate::get_base32_secret(otp)?;
        let period = validate::get_period(otp)?;
        let algorithm = validate::get_algorithm(otp)?;
        let digits = validate::get_digits(otp)?;

        let code = generate_totp(base32_secret, period, algorithm, digits)?;

        if clip {
            clipboard::clip(&code)?;
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
) -> Fallible<String>
where
    S: Into<String>,
{
    let secret = secret.into();
    let auth = TOTPBuilder::new()
        .base32_secret(&secret)?
        .period(period)
        .algorithm(algorithm)
        .output_length(digits)
        .build();
    let code = auth.generate();

    Ok(code)
}

// TODO
#[allow(dead_code)]
fn generate_hotp<S>(
    secret: S,
    counter: u64,
    algorithm: HashAlgorithm,
    digits: usize,
) -> Fallible<String>
where
    S: Into<String>,
{
    let secret = secret.into();
    let auth = HOTPBuilder::new()
        .base32_secret(&secret)?
        .counter(counter)
        .algorithm(algorithm)
        .output_length(digits)
        .build();
    let code = auth.generate();
    // TODO: increment counter, I think?

    Ok(code)
}
