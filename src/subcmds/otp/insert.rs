use failure::Fallible;

pub fn insert(force: bool, echo: bool, pass_name: String, secret: Option<String>) -> Fallible<()> {
    let _ = (force, echo, pass_name, secret);
    Ok(())
}
