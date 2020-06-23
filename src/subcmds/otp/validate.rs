use data_encoding::BASE32_NOPAD;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::otp::HashAlgorithm;
use crate::{PassrsError, Result};

const SCHEME: &str = "otpauth://";
const OTP_TYPE: &str = "(?P<type>totp|hotp)/";
const LABEL: &str = "(?P<label>[^?#]*)";
const SECRET: &str = "(?:[?&]secret=(?P<secret>[^&]*))";
const ISSUER: &str = "(?:[?&]issuer=(?P<issuer>[^&#]*))?";
const ALGORITHM: &str = "(?:[?&]algorithm=(?P<algorithm>[^&#]*))?";
const DIGITS: &str = "(?:[?&]digits=(?P<digits>[^&#]*))?";
const PERIOD: &str = "(?:[?&]period=(?P<period>[^&#]*))?";

static URI_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        &[
            SCHEME, OTP_TYPE, LABEL, ISSUER, ALGORITHM, DIGITS, PERIOD, SECRET,
        ]
        .concat(),
    )
    .expect("Failed to compile OTP URI regex")
});

pub(crate) fn validate<S>(uri: S) -> Result<()>
where
    S: AsRef<str>,
{
    let uri = uri.as_ref();
    let re = &URI_REGEX;

    if let Ok(secret) = self::get_base32_secret(&uri) {
        let secret = secret.to_ascii_uppercase();
        if BASE32_NOPAD.decode(&secret.as_bytes()).is_err() {
            return Err(PassrsError::InvalidKeyUri.into());
        }
    }
    if !re.is_match(&uri) {
        return Err(PassrsError::InvalidKeyUri.into());
    }

    Ok(())
}

pub(crate) fn get_base32_secret<S>(uri: S) -> Result<String>
where
    S: AsRef<str>,
{
    let uri = uri.as_ref();
    let re = &URI_REGEX;
    let captures = re.captures(&uri).ok_or("Failed to get regex captures")?;

    let secret = captures
        .name("secret")
        .ok_or("Failed to get secret from regex")?
        .as_str()
        .to_owned();

    Ok(secret)
}

pub(crate) fn get_period<S>(uri: S) -> Result<u64>
where
    S: AsRef<str>,
{
    let uri = uri.as_ref();
    let re = &URI_REGEX;
    let captures = re.captures(&uri).ok_or("Failed to get regex captures")?;

    let period = match captures.name("period") {
        Some(num) => num.as_str().parse::<u64>()?,
        None => 30,
    };

    Ok(period)
}

pub(crate) fn get_digits<S>(uri: S) -> Result<usize>
where
    S: AsRef<str>,
{
    let uri = uri.as_ref();
    let re = &URI_REGEX;
    let captures = re.captures(&uri).ok_or("Failed to get regex captures")?;

    let digits = match captures.name("digits") {
        Some(num) => num.as_str().parse::<usize>()?,
        None => 6,
    };

    Ok(digits)
}

pub(crate) fn get_algorithm<S>(uri: S) -> Result<HashAlgorithm>
where
    S: AsRef<str>,
{
    let uri = uri.as_ref();
    let re = &URI_REGEX;
    let captures = re.captures(&uri).ok_or("Failed to get regex captures")?;

    let algo = match captures.name("algorithm") {
        Some(algo) => match algo.as_str().to_ascii_lowercase().as_ref() {
            "sha1" => HashAlgorithm::Sha1,
            "sha256" => HashAlgorithm::Sha256,
            "sha512" => HashAlgorithm::Sha512,
            algo => return Err(PassrsError::InvalidHashAlgorithm(algo.to_string()).into()),
        },
        None => HashAlgorithm::Sha1,
    };

    Ok(algo)
}
