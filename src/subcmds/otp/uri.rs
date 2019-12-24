use failure::Fallible;

pub fn uri(clip: bool, qrcode: bool, pass_name: String) -> Fallible<()> {
    let _ = (clip, qrcode, pass_name);
    Ok(())
}
