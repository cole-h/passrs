use std::fs;
use std::os::unix::fs::OpenOptionsExt;

use crate::consts::PASSWORD_STORE_UMASK;
use crate::util;
use crate::{PassrsError, Result};

pub(crate) fn cp(source: String, dest: String, force: bool) -> Result<()> {
    let source_path = util::canonicalize_path(&source)?;
    let is_file = match fs::metadata(&source_path) {
        Ok(meta) => meta.is_file(),
        Err(_) => false,
    };
    let dest_path = if is_file {
        util::exact_path([&dest, ".gpg"].concat())?
    } else {
        util::exact_path(&dest)?
    };

    if is_file {
        if !util::path_exists(&source_path)? {
            return Err(PassrsError::NotInStore(source).into());
        }

        if !force && util::path_exists(&dest_path)? {
            let display = dest_path.display();
            let prompt = format!("An entry exists for {}. Overwrite it?", display);

            if util::prompt_yesno(prompt)? {
                fs::OpenOptions::new()
                    .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
                    .write(true)
                    .truncate(true)
                    .open(&dest_path)?;
            } else {
                return Err(PassrsError::UserAbort.into());
            }
        }

        util::copy(&source_path, &dest_path)?;
        util::commit(
            Some([&source_path, &dest_path]),
            format!("Copy {} to {}", source, dest),
        )?;
    } else {
        if !util::path_exists(&source_path)? {
            return Err(PassrsError::PathDoesntExist(source).into());
        }

        if !force && util::path_exists(&dest_path)? {
            let display = dest_path.display();
            let prompt = format!("An entry exists for {}. Overwrite it?", display);

            if util::prompt_yesno(prompt)? {
                fs::remove_dir_all(&dest_path)?;
            } else {
                return Err(PassrsError::UserAbort.into());
            }
        }

        util::copy(&source_path, &dest_path)?;
        util::commit(
            Some([&source_path, &dest_path]),
            format!("Copy {} to {}", source, dest),
        )?;
    }

    Ok(())
}
