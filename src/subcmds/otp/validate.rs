use failure::Fallible;
use regex::Regex;

use crate::error::PassrsError;

const SCHEME: &str = "otpauth://";
const OTP_TYPE: &str = "(totp|hotp)/";
const LABEL: &str = "([^?#]*)";
const QUERY: &str = "([^#]*)";

pub fn validate(uri: String) -> Fallible<()> {
    let uri_pattern = [SCHEME, OTP_TYPE, LABEL, "\\?", QUERY].concat();
    let re = Regex::new(&uri_pattern)?;

    if re.is_match(&uri) {
        return Ok(());
    }

    //     (SCHEME_PATTERN + "://" + OTP_TYPE_PATTERN + "/" + LABEL_PATTERN + "\\?" + QUERY_PATTERN);

    // if uri.starts_with("otpauth://") {
    //     let uri = uri.split("otpauth://").collect::<String>();
    //     if uri.starts_with("totp/") {
    //         let uri = uri.split("totp/").collect::<String>();
    //         println!("{:?}", uri)
    //     } else if uri.starts_with("hotp/") {
    //         let uri = uri.split("hotp/").collect::<String>();
    //         println!("{:?}", uri)
    //     }
    // }

    Err(PassrsError::InvalidKeyUri.into())
}
