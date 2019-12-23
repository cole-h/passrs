use crate::error::PassrsError;
use crate::Result;

/// Returns `()` if path is valid, or an error if path is invalid.
pub fn verify_path(path: &String) -> Result<()> {
    let meta = std::fs::metadata(path);
    // check if path already exists
    if meta.is_ok() {
        return Err(Box::new(PassrsError::PathExists));
    }
    // TODO: check_sneaky_paths(&path)
    Ok(())
}
