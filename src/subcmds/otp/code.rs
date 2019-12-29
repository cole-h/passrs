// TODO: own impl of TOTP and HOTP to drop transitive 4 deps

use boringauth::oath::{HOTPBuilder, HashFunction, TOTPBuilder};
use failure::{err_msg, Fallible};

use crate::clipboard;
use crate::subcmds::otp::validate;
use crate::ui::{self, UiResult};
use crate::util;

pub fn code(clip: bool, pass_name: String) -> Fallible<()> {
    let file = ui::display_matches(&pass_name)?;

    if let UiResult::Success(file) = file {
        let otp = util::decrypt_file_into_vec(file)?;
        let otp = otp.first().ok_or_else(|| err_msg("Vec was empty"))?;

        // Ensure `otp` is a valid URI
        validate::validate(otp)?;

        // let type = validate::get_type(otp)?;
        // let counter = validate::get_counter(otp)?;
        let base32_secret = validate::get_base32_secret(otp)?;
        let period = validate::get_period(otp)?;
        let algorithm = validate::get_algorithm(otp)?;
        let digits = validate::get_digits(otp)?;

        let code = generate_totp(base32_secret, period, algorithm, digits)?;

        if clip {
            clipboard::clip(&format!("{:0digits$}", code, digits = digits))?;
        } else {
            println!("{:0digits$}", code, digits = digits);
        }
    }

    Ok(())
}

fn generate_totp<S>(secret: S, period: u32, algorithm: HashFunction, digits: usize) -> Fallible<u32>
where
    S: Into<String>,
{
    let secret = secret.into();
    #[allow(deprecated)]
    let auth = TOTPBuilder::new()
        .base32_key(&secret)
        .period(period)
        .hash_function(algorithm)
        .output_len(digits)
        .finalize()
        .unwrap();
    let code = auth.generate().parse::<u32>()?;

    Ok(code)
}

// TODO
#[allow(dead_code)]
fn generate_hotp<S>(
    secret: S,
    counter: u64,
    algorithm: HashFunction,
    digits: usize,
) -> Fallible<u32>
where
    S: Into<String>,
{
    let secret = secret.into();
    let auth = HOTPBuilder::new()
        .base32_key(&secret)
        .counter(counter)
        .hash_function(algorithm)
        .output_len(digits)
        .finalize()
        .unwrap();
    let code = auth.generate().parse::<u32>()?;
    // TODO: increment counter, I think?

    Ok(code)
}
