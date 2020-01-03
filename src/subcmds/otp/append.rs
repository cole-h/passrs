use crate::util;
use crate::Result;

pub fn append(echo: bool, pass_name: String, secret: Option<String>) -> Result<()> {
    // TODO: if pass_name is a folder, write to pass_name/otp
    if echo {
        //
    }
    // TODO: secret
    let _ = secret;

    util::commit(format!("Append OTP secret for {}", pass_name))?;
    Ok(())
}
