use failure::Fallible;

pub fn append(echo: bool, pass_name: String, secret: Option<String>) -> Fallible<String> {
    // TODO: if pass_name is a folder, write to pass_name/otp
    if echo {
        //
    }
    // TODO: secret
    let _ = secret;

    let message = format!("Append OTP secret for {}", pass_name);
    Ok(message)
}
