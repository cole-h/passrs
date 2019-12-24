use failure::Fallible;

pub fn append(echo: bool, pass_name: String, secret: Option<String>) -> Fallible<()> {
    let _ = (echo, pass_name, secret);
    Ok(())
}
