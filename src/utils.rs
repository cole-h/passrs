use failure::Fallible;
use walkdir::WalkDir;

use crate::consts::PASSWORD_STORE_DIR;
use crate::error::PassrsError;

/// Returns `()` if path is valid, or an error if path is invalid.
pub fn verify_path(path: &String) -> Fallible<()> {
    let meta = std::fs::metadata(path);
    // check if path already exists
    if meta.is_ok() {
        return Err(PassrsError::PathExists.into());
    }
    check_sneaky_paths(&path)?;
    Ok(())
}

// TODO: check for .. and shell expansion
fn check_sneaky_paths(path: &String) -> Fallible<()> {
    let _ = path;
    Ok(())
}

// TODO: search in PASSWORD_STORE_DIR for target
pub fn search_entries<S>(target: S) -> Fallible<Vec<String>>
where
    S: Into<String>,
{
    let target = target.into();
    let mut matches = Vec::new();

    for entry in WalkDir::new(&*PASSWORD_STORE_DIR) {
        let mut entry = entry?
            .into_path()
            .to_str()
            .ok_or(failure::err_msg("Path couldn't be converted to string"))?
            .to_string();

        if entry.ends_with(".gpg") {
            entry.truncate(entry.len() - 4);
        }

        if entry.contains(&target) {
            // println!("{}", entry);
            matches.push(entry);
        }
    }

    if matches.len() > 0 {
        Ok(matches)
    } else {
        Err(PassrsError::NoMatchesFound(target).into())
    }
}
