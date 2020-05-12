//! General utilities
//!
//! # util
//!
//! This module houses most of the meat and potatoes of `passrs`. Any generic,
//! helpful function used in more than one place finds its home here.

use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::str;

use anyhow::{Context as _, Result};
use git2::{Commit, Repository};
use gpgme::{Context, Data, Protocol};
use ring::rand;
use termion::input::TermRead;
use walkdir::WalkDir;

use crate::consts::{
    GPG_ID_FILE, HOME, PASSWORD_STORE_DIR, PASSWORD_STORE_KEY, PASSWORD_STORE_UMASK, STORE_LEN,
    STORE_STRING,
};
use crate::PassrsError;

/// Helper function to return the path to the specified entry. Paths may be an
/// absolute path to the entry, or relative to the store's root.
pub fn canonicalize_path<S>(path: S) -> Result<PathBuf>
where
    S: AsRef<str>,
{
    let path = path.as_ref();
    let mut path = path.replace("~", &HOME);

    if !path.contains(&*STORE_STRING) {
        path = [&*STORE_STRING, "/", &path].concat();
    }

    self::check_sneaky_paths(&path)?;

    let enc_path = [&path, ".gpg"].concat();
    let path = if fs::metadata(&enc_path).is_ok() {
        enc_path
    } else if let Ok(meta) = fs::metadata(&path) {
        if meta.is_dir() || path.ends_with('/') {
            path
        } else {
            path + ".gpg"
        }
    } else if path.ends_with('/') {
        path
    } else {
        path + ".gpg"
    };

    Ok(PathBuf::from(path))
}

/// Helper function that ensures that the path is part of the store, but does
/// not assume it exists.
pub fn exact_path<S>(path: S) -> Result<PathBuf>
where
    S: AsRef<str>,
{
    let path = path.as_ref();
    let mut path = path.replace("~", &HOME);

    if !path.contains(&*STORE_STRING) {
        path = [&*STORE_STRING, "/", path.as_str()].concat();
    }

    self::check_sneaky_paths(&path)?;

    Ok(PathBuf::from(path))
}

/// Pretty self explanatory. If neither the specified store directory or
/// `.gpg-id` file exist, the store doesn't exist.
pub fn verify_store_exists() -> Result<()> {
    let store_meta = fs::metadata(&*PASSWORD_STORE_DIR);
    let id_meta = fs::metadata(&*GPG_ID_FILE);

    if store_meta.is_err() || id_meta.is_err() {
        return Err(PassrsError::StoreDoesntExist.into());
    }

    Ok(())
}

/// Returns `false` if path does not exist, `true` if it does exist.
pub fn path_exists<P>(path: P) -> Result<bool>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let meta = fs::metadata(&path);

    if meta.is_ok() {
        return Ok(true);
    }

    self::check_sneaky_paths(path)?;

    Ok(false)
}

/// Paths with `..` are not allowed to prevent changing things outside of the
/// store (or at least aims to).
pub fn check_sneaky_paths<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let path = path.display().to_string();

    if path.contains("../") || path.contains("/..") || path == ".." {
        return Err(PassrsError::SneakyPath(path).into());
    }

    Ok(())
}

/// Search the password store for entries that match the specified `target`.
pub fn find_matches<S>(target: S) -> Result<Vec<String>>
where
    S: AsRef<str>,
{
    let target = target.as_ref();
    let mut matches: Vec<String> = Vec::new();

    for path in WalkDir::new(&*PASSWORD_STORE_DIR)
        .into_iter()
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|s| entry.depth() == 0 || !s.starts_with('.'))
                .unwrap_or(false)
        })
    {
        let entry = path?;
        let is_file = entry.path().is_file();
        let filename = &entry
            .file_name()
            .to_str()
            .with_context(|| "Filename couldn't be converted to str")?;
        let path = entry
            .path()
            .to_str()
            .with_context(|| "Path couldn't be converted to str")?;

        if path.ends_with(".gpg")
            && (filename == &target || target == &filename[..filename.len() - 4])
        {
            return Ok(vec![path.to_owned()]);
        }

        if path.ends_with(".gpg")
            && is_file
            && (path[*STORE_LEN..].contains(target)
                || path[*STORE_LEN..].to_ascii_lowercase().contains(target))
        {
            matches.push(path.to_owned());
        }
    }

    if matches.is_empty() {
        Err(PassrsError::NoMatchesFound(target.to_owned()).into())
    } else {
        matches.sort_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));

        Ok(matches)
    }
}

/// Decrypt the specified file into a `Vec<String>`s. This will return an `Err`
/// if the plaintext is not encoded in valid UTF8.
pub fn decrypt_file_into_strings<P>(path: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let mut bytes = Vec::new();
    let mut file = fs::File::open(path)?;

    file.read_to_end(&mut bytes)?;

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let mut cipher = Data::from_bytes(bytes)?;
    let mut plain = Vec::new();

    ctx.decrypt(&mut cipher, &mut plain)?;

    let plain = str::from_utf8(&plain)?;
    let out = plain.lines().map(ToOwned::to_owned).collect();

    Ok(out)
}

/// Decrypts the specified file into a `Vec<u8>`.
pub fn decrypt_file_into_bytes<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let mut bytes = Vec::new();
    let mut file = fs::File::open(path)?;

    file.read_to_end(&mut bytes)?;

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let mut cipher = Data::from_bytes(bytes)?;
    let mut plain = Vec::new();

    ctx.decrypt(&mut cipher, &mut plain)?;

    Ok(plain)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How to modify the file -- overwrite with new contents, or append new
/// contents to the end.
pub enum EditMode {
    /// Completely overwrite the file.
    Clobber,
    /// Append to the file.
    Append,
}

/// Encrypts a slice of bytes with [`PASSWORD_STORE_KEY`] or the key(s) listed
/// in `.gpg-id`. Callers must verify that [`PASSWORD_STORE_DIR`] exists and is
/// initialized using `verify_store_exists`. If `editmode` is `Append`, append
/// the bytes; otherwise, overwrite the file.
///
/// [`PASSWORD_STORE_KEY`]: ../consts/static.PASSWORD_STORE_KEY.html
/// [`PASSWORD_STORE_DIR`]: ../consts/static.PASSWORD_STORE_DIR.html
pub fn encrypt_bytes_into_file<P, V>(to_encrypt: V, path: P, editmode: EditMode) -> Result<()>
where
    P: AsRef<Path>,
    V: AsRef<[u8]>,
{
    let to_encrypt = to_encrypt.as_ref();
    let path = path.as_ref();

    self::create_dirs_to_file(&path)?;

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let id_file = match self::find_gpg_id(
        path.parent()
            .with_context(|| format!("{} didn't have a parent", path.display()))?,
    ) {
        Ok(gpgid) => fs::OpenOptions::new().read(true).open(&gpgid)?,
        Err(_) => fs::OpenOptions::new().read(true).open(&*GPG_ID_FILE)?,
    };
    let reader = BufReader::new(&id_file);
    let mut keys = Vec::new();

    for line in reader.lines() {
        keys.push(line?);
    }

    keys.extend(PASSWORD_STORE_KEY.clone());

    let encryption_keys: Vec<gpgme::Key> = keys
        .iter()
        .map(|k| ctx.get_secret_key(k))
        .filter_map(|k| k.ok())
        .collect();

    if encryption_keys.is_empty() {
        return Err(PassrsError::NoSigningKeyFound.into());
    } else {
        let mut to_be_encrypted: Vec<u8> = Vec::new();

        if editmode == EditMode::Append {
            let mut to_be_decrypted = Data::load(path.display().to_string())?;

            ctx.decrypt(&mut to_be_decrypted, &mut to_be_encrypted)?;
            to_be_encrypted.push(b'\n');
        }

        to_be_encrypted.extend(to_encrypt);

        let mut encrypted_contents: Vec<u8> = Vec::new();
        let mut file = fs::OpenOptions::new()
            .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
            .write(true)
            .create(true)
            .open(&path)?;

        ctx.encrypt(
            &encryption_keys,
            &mut to_be_encrypted,
            &mut encrypted_contents,
        )?;
        file.write_all(&encrypted_contents)?;
    }

    Ok(())
}

/// A light wrapper around [`fs::create_dir_all`] that creates all directories
/// to allow the specified `file` to be created.
///
/// [`fs::create_dir_all`]: https://doc.rust-lang.org/std/fs/fn.create_dir_all.html
pub fn create_dirs_to_file<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    self::check_sneaky_paths(&path)?;

    if path.exists() {
        return Ok(());
    }

    let dir = path
        .parent()
        .with_context(|| format!("Path '{}' had no parent", path.display()))?;

    fs::create_dir_all(dir)?;
    self::set_permissions_recursive(&path)?;

    Ok(())
}

/// Analogous to coreutils' `rmdir -p` -- delete directories down to the
/// specified path unless the directory is not empty, in which case we stop.
pub fn remove_dirs_to_file<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    self::check_sneaky_paths(&path)?;

    if !path.exists() {
        return Ok(());
    }

    if path.is_file() {
        fs::remove_file(&path)?;
    }

    let mut dir = path;
    while let Some(new_dir) = dir.parent() {
        dir = new_dir;
        let is_empty = dir
            .read_dir()
            .map(|mut i| i.next().is_none())
            .unwrap_or(false);

        if is_empty {
            assert_ne!(dir, *PASSWORD_STORE_DIR);
            fs::remove_dir(dir)?;
        }
    }

    Ok(())
}

/// Change the permissions of every file and directory specified, using
/// [`PASSWORD_STORE_UMASK`], unless it does not belong to the user.
///
/// [`PASSWORD_STORE_UMASK`]: ../consts/static.PASSWORD_STORE_UMASK.html
pub fn set_permissions_recursive<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    self::check_sneaky_paths(&path)?;

    let uid = unsafe { libc::getuid() };
    let path_uid = if path.exists() {
        path.metadata()?.uid()
    } else {
        uid
    };

    // Prevent changes to any path the user doesn't own by comparing uids,
    // because that would error.
    if path_uid != uid {
        return Ok(());
    }

    if path.is_dir() {
        let mut perms = fs::metadata(&path)
            .with_context(|| format!("Path {} does not exist", path.display()))?
            .permissions();
        perms.set_mode(perms.mode() - (perms.mode() & *PASSWORD_STORE_UMASK));

        fs::set_permissions(&path, perms)
            .with_context(|| format!("Failed to set permissions for '{}'", path.display()))?;

        if path == *PASSWORD_STORE_DIR {
            return Ok(());
        } else {
            self::set_permissions_recursive(
                path.parent()
                    .with_context(|| format!("Path '{}' had no parent", path.display()))?,
            )?;
        }
    } else {
        self::set_permissions_recursive(
            path.parent()
                .with_context(|| format!("Path '{}' had no parent", path.display()))?,
        )?;
    }

    Ok(())
}

/// Find a `.gpg-id` file in the specified path and return it if found;
/// otherwise, return an error.
pub fn find_gpg_id<P>(path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let show = file_name
            .to_str()
            .map(|e| !e.starts_with(".git"))
            .unwrap_or(false);

        if show && file_name == ".gpg-id" {
            return Ok(path);
        }
    }

    Err(PassrsError::NoGpgIdFile(path.display().to_string()).into())
}

/// Commit all modified files using `commit_message` as the message, if the
/// workspace is dirty. If a path is specified, also add that path to the git
/// repository.
///
/// [`PASSWORD_STORE_DIR`]: ../consts/static.PASSWORD_STORE_DIR.html
pub fn commit<S, P, V>(paths: Option<V>, commit_message: S) -> Result<()>
where
    S: AsRef<str>,
    V: AsRef<[P]>,
    P: AsRef<Path>,
{
    let commit_message = commit_message.as_ref();

    // NOTE: similarly implemented in subcmds/init.rs
    if let Ok(repo) = Repository::open(&*PASSWORD_STORE_DIR) {
        if repo.statuses(None)?.is_empty() {
            writeln!(io::stdout(), "Nothing to do")?;
            return Ok(());
        }

        let mut index = repo.index()?;
        let config = repo.config()?;
        let sig = repo.signature()?;

        if let Some(paths) = paths {
            let mut pathspecs = Vec::new();

            for path in paths.as_ref() {
                let path = path.as_ref();
                let path = if path.starts_with(&*STORE_STRING) {
                    PathBuf::from(&path.display().to_string()[*STORE_LEN..])
                } else {
                    path.to_path_buf()
                };

                pathspecs.push(path);
            }

            index.add_all(pathspecs, git2::IndexAddOption::CHECK_PATHSPEC, None)?;
        } else {
            index.update_all(&["."], None)?;
        }

        index.write()?;

        let tree_id = repo.index()?.write_tree()?;
        let mut parents = Vec::new();

        if let Some(parent) = repo
            .head()
            .ok()
            .map(|h| h.target().expect("HEAD had no target"))
        {
            parents.push(repo.find_commit(parent)?);
        }

        let mut status_opts = git2::StatusOptions::new();

        status_opts.renames_head_to_index(true);

        let statuses = repo.statuses(Some(&mut status_opts))?;
        let parents: Vec<&Commit> = parents.iter().collect();
        let commit = if config.get_bool("commit.gpgsign")? {
            let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
            let buf = repo.commit_create_buffer(
                &sig,
                &sig,
                &commit_message,
                &repo.find_tree(tree_id)?,
                &parents,
            )?;
            let mut outbuf = Vec::new();

            ctx.set_armor(true);
            ctx.sign_detached(&*buf, &mut outbuf)?;

            let contents = buf.as_str().with_context(|| "Buffer was not valid UTF-8")?;
            let out = str::from_utf8(&outbuf)?;

            repo.commit_signed(&contents, &out, Some("gpgsig"))?
        } else {
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &commit_message,
                &repo.find_tree(tree_id)?,
                &parents,
            )?
        };
        let head = repo.head()?;
        let branch = head.shorthand().unwrap_or("master");

        repo.reference(
            &format!("refs/heads/{}", branch),
            commit,
            true,
            &commit_message,
        )?;

        let mut diff = repo.diff_tree_to_tree(
            repo.revparse_single("HEAD^")?
                .peel(git2::ObjectType::Tree)?
                .as_tree(),
            repo.revparse_single("HEAD")?
                .peel(git2::ObjectType::Tree)?
                .as_tree(),
            None,
        )?;
        diff.find_similar(None)?;

        let stats = diff.stats()?;
        let buf = stats.to_buf(git2::DiffStatsFormat::SHORT, 80)?;

        writeln!(
            io::stdout(),
            "[{} {:.7}] {}",
            branch,
            commit,
            commit_message
        )?;
        write!(
            io::stdout(),
            "{}",
            str::from_utf8(&*buf).with_context(|| "diffstats should be valid utf8")?
        )?;

        for entry in statuses
            .iter()
            .filter(|e| e.status() != git2::Status::CURRENT)
        {
            // NOTE: rename detection doesn't work for recrypted files/dirs
            let index_status = match entry.status() {
                s if s.contains(git2::Status::INDEX_NEW) => "create",
                s if s.contains(git2::Status::INDEX_DELETED) => "delete",
                s if s.contains(git2::Status::INDEX_RENAMED) => "rename",
                s if s.contains(git2::Status::INDEX_MODIFIED) => "rewrite",
                _ => continue,
            };
            let old_path = entry
                .head_to_index()
                .with_context(|| "couldn't get differences between HEAD and index")?
                .old_file()
                .path();
            let new_path = entry
                .head_to_index()
                .with_context(|| "couldn't get differences between HEAD and index")?
                .new_file()
                .path();

            // FIXME: similarity is not yet exposed in git2-rs
            //   https://github.com/rust-lang/git2-rs/blob/7f076f65a8ceb8dd1f8baa627982760132fdd2e9/src/diff.rs#L387
            match (old_path, new_path) {
                (Some(old), Some(new)) if old != new => {
                    let percent_change = 100;

                    writeln!(
                        io::stdout(),
                        " {} {} => {} ({}%)",
                        index_status,
                        old.display(),
                        new.display(),
                        percent_change
                    )?;
                }
                (Some(old), Some(_)) if index_status == "rewrite" => {
                    let percent_change = 100;

                    writeln!(
                        io::stdout(),
                        " {} {} ({}%)",
                        index_status,
                        old.display(),
                        percent_change
                    )?;
                }
                (old, new) => {
                    let path = old
                        .or(new)
                        .with_context(|| "neither old nor new were valid paths")?;
                    let tree = repo.find_tree(tree_id)?;
                    let file = tree.iter().find(|e| e.name() == path.to_str());
                    let mode = match file {
                        Some(file) => file.filemode() as u32,
                        None => 0o100_644,
                    };

                    writeln!(
                        io::stdout(),
                        " {} mode {:o} {}",
                        index_status,
                        mode,
                        path.display()
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Provided a `Vec<u8>` of characters and a length, randomly generate a
/// password.
///
/// This uses the [ring] crate to generate an array of a multiple of 64 bytes
/// for use as random indices.
///
/// [ring]: https://docs.rs/ring
pub fn generate_chars_from_set<V>(set: V, len: usize) -> Result<Vec<u8>>
where
    V: AsRef<[u8]>,
{
    let set = set.as_ref();
    let rng = rand::SystemRandom::new();
    let mut secret_bytes: Vec<u8> = Vec::with_capacity(len);
    let mut random: Vec<u8> = Vec::new();

    for _ in 0..=(len / 64) {
        let rand: [u8; 64] = rand::generate(&rng)
            .expect("failed to generate random array")
            .expose();

        random.extend(rand.iter());
    }

    for &rand in random.iter() {
        secret_bytes.push(set[rand as usize % set.len()]);

        if secret_bytes.len() == len {
            break;
        }
    }

    assert_eq!(secret_bytes.len(), len);

    Ok(secret_bytes)
}

/// Helper function to prompt the user for a secret to be inserted or appended
/// or otherwise added to the store, handling the bool or multiline flags as
/// needed.
pub fn prompt_for_secret<S>(secret_name: S, echo: bool, multiline: bool) -> Result<Option<String>>
where
    S: AsRef<str>,
{
    let secret_name = secret_name.as_ref();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let secret = if echo {
        write!(io::stdout(), "Enter secret for {}: ", secret_name)?;
        io::stdout().flush()?;
        let input = TermRead::read_line(&mut stdin)?;

        if input.is_none() {
            return Err(PassrsError::UserAbort.into());
        }

        input
    } else if multiline {
        write!(
            io::stdout(),
            "Enter the contents of {} and press Ctrl-D when finished:\n\n",
            secret_name
        )?;
        let mut input = Vec::new();

        for line in stdin.lines() {
            input.push(line?);
        }

        Some(input.join("\n"))
    } else {
        write!(io::stdout(), "Enter secret for {}: ", secret_name)?;
        io::stdout().flush()?;
        let input = {
            let input = stdin.read_passwd(&mut io::stdout())?;
            writeln!(io::stdout())?;

            if let Some(input) = input {
                input
            } else {
                return Err(PassrsError::UserAbort.into());
            }
        };

        write!(io::stdout(), "Re-enter secret for {}: ", secret_name)?;
        io::stdout().flush()?;
        let check = {
            let input = stdin.read_passwd(&mut io::stdout())?;
            writeln!(io::stdout())?;

            if let Some(input) = input {
                input
            } else {
                return Err(PassrsError::UserAbort.into());
            }
        };

        if input == check {
            Some(input)
        } else {
            return Err(PassrsError::SecretsDontMatch.into());
        }
    };

    Ok(secret)
}

/// Helper function to ask the user whether or not they really wanted to ____
/// (as specified by the `prompt`). As long as the response starts with the
/// letter `y` (case insensitive), the reply is treated as affirmative.
pub fn prompt_yesno<S>(prompt: S) -> Result<bool>
where
    S: AsRef<str>,
{
    let prompt = prompt.as_ref();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    write!(io::stdout(), "{} [y/N] ", prompt)?;
    io::stdout().flush()?;

    match TermRead::read_line(&mut stdin)? {
        Some(ref reply) if reply.to_ascii_lowercase().starts_with('y') => Ok(true),
        _ => Ok(false),
    }
}

/// Helper function to recrypt a directory that handles the case where a
/// subdirectory has a `.gpg-id` (which causes it to break out of the loop, thus
/// ignoring any directory with a `.gpg-id` except for the root,
/// [`PASSWORD_STORE_DIR`]). If no `keys` are specified, use the key(s) in the
/// closest `.gpg-id`.
///
/// [`PASSWORD_STORE_DIR`]: ../consts/static.PASSWORD_STORE_DIR.html
pub fn recrypt_dir<P>(path: P, keys: Option<&[String]>) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let keys = if let Some(keys) = keys {
        Vec::from(keys)
    } else {
        let gpgid = self::get_closest_gpg_id(&path)?;
        let mut keys = Vec::new();
        let mut file = fs::OpenOptions::new().read(true).open(&gpgid)?;

        file.read_to_end(&mut keys)?;

        let keys = str::from_utf8(&keys)?;

        keys.lines().map(ToOwned::to_owned).collect()
    };

    if keys.is_empty() {
        return Err(PassrsError::NoPrivateKeyFound.into());
    }

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name();
        let show = entry
            .file_name()
            .to_str()
            .map(|e| !e.starts_with(".git"))
            .unwrap_or(false);

        if show {
            if file_name == Some(".gpg-id".as_ref()) {
                if path == *GPG_ID_FILE {
                    continue;
                } else {
                    break;
                }
            }

            if path.is_file() && path.extension() == Some("gpg".as_ref()) {
                self::recrypt_file(path, Some(&keys))?;
            } else if path.is_dir() {
                self::recrypt_dir(path, Some(&keys))?;
            }
        }
    }

    Ok(())
}

/// Recrypts an individual file at `path` with `keys` (or [`PASSWORD_STORE_KEY`]
/// if no keys are specified).
///
/// [`PASSWORD_STORE_DIR`]: ../consts/static.PASSWORD_STORE_DIR.html
pub fn recrypt_file<S>(path: S, keys: Option<&[String]>) -> Result<()>
where
    S: AsRef<Path>,
{
    let path = path.as_ref();
    let keys = if let Some(keys) = keys {
        Vec::from(keys)
    } else {
        let gpgid = self::get_closest_gpg_id(&path)?;
        let mut keys = Vec::new();
        let mut file = fs::OpenOptions::new().read(true).open(&gpgid)?;

        file.read_to_end(&mut keys)?;

        let keys = str::from_utf8(&keys)?;

        keys.lines().map(ToOwned::to_owned).collect()
    };

    if keys.is_empty() {
        return Err(PassrsError::NoPrivateKeyFound.into());
    }

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let keys: Vec<gpgme::Key> = keys
        .iter()
        .map(|k| ctx.get_secret_key(k))
        .filter_map(|k| k.ok())
        .collect();

    if keys.is_empty() {
        return Err(PassrsError::NoPrivateKeyFound.into());
    }

    let mut encrypted_contents = Data::load(path.display().to_string())?;
    let mut decrypted_contents = Vec::new();

    ctx.decrypt(&mut encrypted_contents, &mut decrypted_contents)?;

    let mut file = fs::OpenOptions::new()
        .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
        .write(true)
        .open(&path)?;
    let mut encrypted_contents = Vec::new();

    ctx.encrypt(&keys, &decrypted_contents, &mut encrypted_contents)?;
    file.write_all(&encrypted_contents)?;

    Ok(())
}

/// Helper function to find the `.gpg-id` closest to the specified path.
pub fn get_closest_gpg_id<P>(path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let path = if path.is_file() {
        path.parent()
            .with_context(|| "file's parent doesn't exist")?
    } else {
        path
    };

    if path == *PASSWORD_STORE_DIR {
        return Ok(GPG_ID_FILE.clone());
    }

    match self::find_gpg_id(&path) {
        Ok(gpgid) => Ok(gpgid),
        Err(_) => self::get_closest_gpg_id(
            path.parent()
                .with_context(|| "path's parent doesn't exist")?,
        ),
    }
}

/// Some subcommands require user interaction, which in turn requires stdout is
/// a tty.
pub fn ensure_stdout_is_tty() -> Result<()> {
    if termion::is_tty(&io::stdout()) {
        Ok(())
    } else {
        Err(PassrsError::StdoutNotTty.into())
    }
}

/// Copies `source` to `dest` recursively.
pub fn copy<P, Q>(source: &P, dest: &Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    self::copy_impl(source, dest, None)
}

// https://github.com/mdunsmuir/copy_dir/blob/071bab19cd716825375e70644c080c36a58863a1/src/lib.rs#L118
// Original work Copyright (c) 2016 Michael Dunsmuir
// Modified work Copyright (c) 2019 Cole Helbling
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
fn copy_impl<P, Q>(source: &P, dest: &Q, mut root: Option<(u64, u64)>) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    fn uid(path: &Path) -> Result<(u64, u64)> {
        let metadata = path.metadata()?;
        Ok((metadata.dev(), metadata.ino()))
    }

    let source = source.as_ref();
    let dest = dest.as_ref();
    let id = uid(source)?;
    let meta = source.metadata()?;

    if meta.is_file() {
        if fs::metadata(&dest).is_err() {
            self::create_dirs_to_file(&dest)?;
        }

        fs::copy(source, dest)?;
    } else if meta.is_dir() {
        if let Some(root) = root {
            if root == id {
                return Err(PassrsError::SourceIsDestination.into());
            }
        }

        fs::create_dir_all(&dest)?;

        if root.is_none() {
            root = Some(uid(&dest)?);
        }

        for entry in fs::read_dir(source)? {
            let entry = entry?.path();
            let name = entry
                .file_name()
                .with_context(|| "Entry did not contain valid filename")?;
            self::copy_impl(&entry, &dest.join(name), root)?;
        }

        fs::set_permissions(dest, meta.permissions())?;
    } else {
        // not file or dir (probably -> doesn't exist)
    }

    Ok(())
}
