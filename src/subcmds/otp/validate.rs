use anyhow::{Context, Result};
use data_encoding::BASE32_NOPAD;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::otp::HashAlgorithm;
use crate::PassrsError;

const SCHEME: &str = "otpauth://";
const OTP_TYPE: &str = "(?P<type>totp|hotp)/";
const LABEL: &str = "(?P<label>[^?#]*)";
const SECRET: &str = "(?:\\?secret=(?P<secret>[^&]*))";
const ISSUER: &str = "(?:&issuer=(?P<issuer>[^&#]*))?";
const ALGORITHM: &str = "(?:&algorithm=(?P<algorithm>[^&#]*))?";
const DIGITS: &str = "(?:&digits=(?P<digits>[^&#]*))?";
const PERIOD: &str = "(?:&period=(?P<period>[^&#]*))?";

static URI_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        &[
            SCHEME, OTP_TYPE, LABEL, SECRET, ISSUER, ALGORITHM, DIGITS, PERIOD,
        ]
        .concat(),
    )
    .unwrap()
});

pub fn validate<S>(uri: S) -> Result<()>
where
    S: AsRef<str>,
{
    let re = &URI_REGEX;
    let uri = uri.as_ref();

    // if secret is not base32, error
    if let Ok(secret) = self::get_base32_secret(&uri) {
        if let Err(err) = BASE32_NOPAD.decode(&secret.as_bytes()) {
            return Err(PassrsError::InvalidKeyUri.into());
        }
    }
    if !re.is_match(&uri) {
        return Err(PassrsError::InvalidKeyUri.into());
    }

    Ok(())
}

pub fn get_base32_secret<S>(uri: S) -> Result<String>
where
    S: AsRef<str>,
{
    let re = &URI_REGEX;
    let uri = uri.as_ref();

    let captures = re
        .captures(&uri)
        .with_context(|| "Failed to get regex captures")?;
    let secret = captures
        .name("secret")
        .with_context(|| "Failed to get secret from regex")?
        .as_str()
        .to_owned();

    Ok(secret)
}

pub fn get_period<S>(uri: S) -> Result<u64>
where
    S: AsRef<str>,
{
    let re = &URI_REGEX;
    let uri = uri.as_ref();

    let captures = re
        .captures(&uri)
        .with_context(|| "Failed to get regex captures")?;
    let period = match captures.name("period") {
        Some(num) => num.as_str().parse::<u64>()?,
        None => 30,
    };

    Ok(period)
}

pub fn get_digits<S>(uri: S) -> Result<usize>
where
    S: AsRef<str>,
{
    let re = &URI_REGEX;
    let uri = uri.as_ref();

    let captures = re
        .captures(&uri)
        .with_context(|| "Failed to get regex captures")?;
    let digits = match captures.name("digits") {
        Some(num) => num.as_str().parse::<usize>()?,
        None => 6,
    };

    Ok(digits)
}

#[allow(deprecated)]
pub fn get_algorithm<S>(uri: S) -> Result<HashAlgorithm>
where
    S: AsRef<str>,
{
    let re = &URI_REGEX;
    let uri = uri.as_ref();

    let captures = re
        .captures(&uri)
        .with_context(|| "Failed to get regex captures")?;
    let algo = match captures.name("algorithm") {
        Some(algo) => match algo.as_str().to_lowercase().as_ref() {
            "sha1" => HashAlgorithm::Sha1,
            "sha256" => HashAlgorithm::Sha256,
            "sha512" => HashAlgorithm::Sha512,
            algo => return Err(PassrsError::InvalidHashAlgorithm(algo.to_owned()).into()),
        },
        None => HashAlgorithm::Sha1,
    };

    Ok(algo)
}
