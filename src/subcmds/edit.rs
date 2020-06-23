use std::env;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

use anyhow::{Context, Result};
use data_encoding::HEXLOWER;
use ring::digest;

use crate::consts::{EDITOR, PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS, PASSWORD_STORE_UMASK};
use crate::util;
use crate::util::EditMode;
use crate::PassrsError;

pub(crate) fn edit<S>(secret_name: S) -> Result<()>
where
    S: AsRef<str>,
{
    let secret_name = secret_name.as_ref();

    // 1. Decrypt file to /dev/shm/{exe}.{20 rand alnum chars}/{5 rand
    // alnum}-path-components-except-for-root.txt
    let temp_path = self::temp_file(&secret_name)?;
    let file = util::canonicalize_path(&secret_name)?;

    util::create_dirs_to_file(&temp_path)?;

    // If file doesn't exist, create empty Vec for contents (we can't decrypt a
    // nonexistent file). This is the cleanest solution without throwing
    // conditionals everywhere to ensure the file exists.
    let contents = if file.is_file() {
        util::decrypt_file_into_bytes(&file)?
    } else {
        Vec::new()
    };
    let hash = HEXLOWER.encode(digest::digest(&digest::SHA256, &contents).as_ref());

    let mut temp_file = fs::OpenOptions::new()
        .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
        .write(true)
        .create(true)
        .open(&temp_path)?;

    temp_file.write_all(&contents)?;

    // 2. Spawn editor of that file
    Command::new(&*EDITOR)
        .arg(&temp_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        // 3. Wait for process to exit
        .status()?;

    // 4. Read new contents of the tempfile and calculate a hash of the contents
    let mut new_contents = Vec::new();
    let mut temp_file = fs::OpenOptions::new()
        .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
        .read(true)
        .open(&temp_path)?;

    temp_file.seek(SeekFrom::Start(0))?;
    temp_file.read_to_end(&mut new_contents)?;

    let new_hash = HEXLOWER.encode(digest::digest(&digest::SHA256, &new_contents).as_ref());

    // 5a. If the hashes are different but the user didn't enter anything, treat
    // that as a user abort
    if new_contents.is_empty() {
        fs::remove_file(&temp_path)?;
        fs::remove_dir(
            PathBuf::from(&temp_path)
                .parent()
                .with_context(|| "Path did not contain a parent")?,
        )?;

        return Err(PassrsError::UserAbort.into());
    // 5b. If the hashes are the same, nothing changed
    } else if hash == new_hash {
        fs::remove_file(&temp_path)?;
        fs::remove_dir(
            PathBuf::from(&temp_path)
                .parent()
                .with_context(|| "Path did not contain a parent")?,
        )?;

        return Err(PassrsError::ContentsUnchanged.into());
    }

    // 6. Encrypt contents of temp_file to file in store
    util::encrypt_bytes_into_file(&new_contents, &file, EditMode::Clobber)?;
    util::commit(
        Some([&file]),
        format!("Edit secret for {} using {}", secret_name, *EDITOR),
    )?;

    // 7. delete temporary file and directory
    fs::remove_file(&temp_path)?;
    fs::remove_dir(
        PathBuf::from(&temp_path)
            .parent()
            .with_context(|| "Path did not contain a parent")?,
    )?;

    Ok(())
}

fn temp_file<S>(path: S) -> Result<String>
where
    S: AsRef<str>,
{
    let path = path.as_ref();
    let path = path.replace("/", "-");
    let folder = util::generate_chars_from_set(&*PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS, 20)?;
    let file = util::generate_chars_from_set(&*PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS, 5)?;

    let path = format!(
        "{dir}/{exe}.{folder}/{file}-{path}.txt",
        dir = if PathBuf::from("/dev/shm").metadata().is_ok() {
            String::from("/dev/shm")
        } else {
            let prompt = "Your system does not have /dev/shm, which means that it may\n\
                 be difficult to securely erase the temporary non-encrypted\n\
                 password file after editing.\n\
                 Are you sure you would like to continue?";

            if !util::prompt_yesno(prompt)? {
                return Err(PassrsError::UserAbort.into());
            }

            env::var("TMPDIR").unwrap_or_else(|_| String::from("/tmp"))
        },
        exe = env::current_exe()?
            .file_name()
            .with_context(|| "Current executable doesn't have a filename...?")?
            .to_string_lossy(),
        folder = str::from_utf8(&folder)?,
        file = str::from_utf8(&file)?,
        path = path
    );

    Ok(path)
}
