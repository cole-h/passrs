use boringauth::oath::HashFunction;
use failure::{err_msg, Fallible};
use lazy_static::lazy_static;
use regex::Regex;

use crate::error::PassrsError;

const SCHEME: &str = "otpauth://";
const OTP_TYPE: &str = "(?P<type>totp|hotp)/";
const LABEL: &str = "(?P<label>[^?#]*)";
const SECRET: &str = "(?:\\?secret=(?P<secret>[^&]*))";
const ISSUER: &str = "(?:&issuer=(?P<issuer>[^&#]*))?";
const ALGORITHM: &str = "(?:&algorithm=(?P<algorithm>[^&#]*))?";
const DIGITS: &str = "(?:&digits=(?P<digits>[^&#]*))?";
const PERIOD: &str = "(?:&period=(?P<period>[^&#]*))?";

lazy_static! {
    static ref URI_PATTERN: String =
        [SCHEME, OTP_TYPE, LABEL, SECRET, ISSUER, ALGORITHM, DIGITS, PERIOD].concat();
}

pub fn validate<S>(uri: S) -> Fallible<()>
where
    S: Into<String>,
{
    let re = Regex::new(&URI_PATTERN)?;
    let uri = uri.into();

    if re.is_match(&uri) {
        return Ok(());
    }

    Err(PassrsError::InvalidKeyUri.into())
}

pub fn get_base32_secret<S>(uri: S) -> Fallible<String>
where
    S: Into<String>,
{
    let re = Regex::new(&URI_PATTERN)?;
    let uri = uri.into();

    let captures = re
        .captures(&uri)
        .ok_or_else(|| err_msg("Failed to get regex captures"))?;
    let secret = captures
        .name("secret")
        .ok_or_else(|| err_msg("Failed to get secret from regex"))?
        .as_str()
        .to_owned();

    Ok(secret)
}

pub fn get_period<S>(uri: S) -> Fallible<u32>
where
    S: Into<String>,
{
    let re = Regex::new(&URI_PATTERN)?;
    let uri = uri.into();

    let captures = re
        .captures(&uri)
        .ok_or_else(|| err_msg("Failed to get regex captures"))?;
    let period = match captures.name("period") {
        Some(num) => num.as_str().parse::<u32>()?,
        None => 30,
    };

    Ok(period)
}

pub fn get_digits<S>(uri: S) -> Fallible<usize>
where
    S: Into<String>,
{
    let re = Regex::new(&URI_PATTERN)?;
    let uri = uri.into();

    let captures = re
        .captures(&uri)
        .ok_or_else(|| err_msg("Failed to get regex captures"))?;
    let digits = match captures.name("digits") {
        Some(num) => num.as_str().parse::<usize>()?,
        None => 6,
    };

    Ok(digits)
}

#[allow(deprecated)]
pub fn get_algorithm<S>(uri: S) -> Fallible<HashFunction>
where
    S: Into<String>,
{
    let re = Regex::new(&URI_PATTERN)?;
    let uri = uri.into();

    let captures = re
        .captures(&uri)
        .ok_or_else(|| err_msg("Failed to get regex captures"))?;
    let algo = match captures.name("algorithm") {
        Some(algo) => match algo.as_str().to_lowercase().as_ref() {
            "sha1" => HashFunction::Sha1,
            "sha256" => HashFunction::Sha256,
            "sha512" => HashFunction::Sha512,
            algo => return Err(PassrsError::InvalidHashFunction(algo.to_owned()).into()),
        },
        None => HashFunction::Sha1,
    };

    Ok(algo)
}
