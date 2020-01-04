use std::env;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

use anyhow::Result;
use data_encoding::HEXLOWER;
use ring::digest;
use zeroize::Zeroize;

use crate::consts::{EDITOR, PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS};
use crate::util;
use crate::PassrsError;

pub fn edit(pass_name: String) -> Result<()> {
    // 1. decrypt file to /dev/shm/{exe}.{20 rand alnum chars}/{5 rand
    // alnum}-path-components-except-for-root.txt
    let temp_path = temp_file(&pass_name)?;
    let file = util::canonicalize_path(&pass_name)?;

    util::create_descending_dirs(&temp_path)?;

    let mut contents = util::decrypt_file_into_bytes(&file)?;
    let hash = HEXLOWER.encode(digest::digest(&digest::SHA256, &contents).as_ref());
    let mut tempfile = fs::OpenOptions::new()
        .mode(0o600)
        .read(true)
        .write(true)
        .create_new(true)
        .open(&temp_path)?;

    tempfile.write_all(&contents)?;
    contents.zeroize();

    // 2. spawn editor of that file
    Command::new(&*EDITOR)
        .arg(&temp_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        // 3. wait for process to exit
        .output()?;

    // 4. read new contents of the tempfile and calculate a hash of the contents
    let mut new_contents = Vec::new();

    tempfile.seek(SeekFrom::Start(0))?;
    tempfile.read_to_end(&mut new_contents)?;

    let new_hash = HEXLOWER.encode(digest::digest(&digest::SHA256, &new_contents).as_ref());

    // 5a. if same, zero_memory both and notify nothing changed
    // 5b. if not same, truncate old file and write new bytes to file
    if hash == new_hash {
        new_contents.zeroize();
        fs::remove_file(&temp_path)?;
        fs::remove_dir(PathBuf::from(&temp_path).parent().unwrap())?;

        return Err(PassrsError::ContentsUnchanged.into());
    } else if new_contents.is_empty() {
        new_contents.zeroize();
        fs::remove_file(&temp_path)?;
        fs::remove_dir(PathBuf::from(&temp_path).parent().unwrap())?;

        return Err(PassrsError::UserAbort.into());
    }

    // 6. encrypt contents of /dev/shm to file in store
    util::encrypt_bytes_into_file(&new_contents, &temp_path)?;
    new_contents.zeroize();

    // 7. delete temporaries
    fs::remove_file(&temp_path)?;
    fs::remove_dir(PathBuf::from(&temp_path).parent().unwrap())?;

    util::commit(format!("Edit secret for {} using {}", pass_name, *EDITOR))?;
    Ok(())
}

fn temp_file<S>(path: S) -> Result<String>
where
    S: AsRef<str>,
{
    assert!(PathBuf::from("/dev/shm/").metadata().is_ok());

    let path = path.as_ref();

    let path = path.replace("/", "-");
    let folder = util::generate_chars_from_set(&*PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS, 20)?;
    let file = util::generate_chars_from_set(&*PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS, 5)?;

    let path = format!(
        "/dev/shm/{exe}.{folder}/{file}-{path}.txt",
        exe = env::current_exe()?.file_name().unwrap().to_string_lossy(),
        folder = str::from_utf8(&folder)?,
        file = str::from_utf8(&file)?,
        path = path
    );

    Ok(path)
}
