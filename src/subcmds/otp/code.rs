use failure::Fallible;

pub fn code(clip: bool, pass_name: String) -> Fallible<()> {
    let _ = (clip, pass_name);
    Ok(())
}
