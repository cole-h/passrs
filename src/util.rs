use failure::{err_msg, Fallible};
use git2::Repository;
use gpgme::{Context, Data, Protocol, SignMode};

use crate::consts::{HOME, PASSWORD_STORE_DIR, PASSWORD_STORE_KEY, PASSWORD_STORE_SIGNING_KEY};
use crate::error::PassrsError;

// TODO: check paths in every function that reads or writes to .password-store
// TODO: move all git-related fns to a git.rs
// TODO: all functions except init require that the password store exists
// (git repo must be init'd, and .gpg-id must be present)

// TODO: for all commands that write to the store, create all dirs (rfind('/')
// and go from there)
// 1. canonicalize_path
// 2. path_exists (which implicitly check_sneaky_paths)
// 3. sep=rfind('/') and fs::create_all_dirs(path[..sep])

// TODO: verify function doesn't munge the path
/// Paths may be an absolute path to the entry, or relative to the store's root.
pub fn canonicalize_path(path: &str) -> Fallible<String> {
    let mut path = path.replace("~", &*HOME);

    if !path.contains(&*PASSWORD_STORE_DIR) {
        path = [&*PASSWORD_STORE_DIR.to_owned(), &path].concat();
    }

    Ok(path)
}

pub fn verify_store_exists() -> Fallible<()> {
    let meta = std::fs::metadata(&*PASSWORD_STORE_DIR);
    if meta.is_err() {
        return Err(PassrsError::StoreDoesntExist.into());
    }

    let gpg_id = [&*PASSWORD_STORE_DIR, ".gpg-id"].concat();
    let meta = std::fs::metadata(gpg_id);
    if meta.is_err() {
        return Err(PassrsError::StoreDoesntExist.into());
    }

    Ok(())
}

/// Returns `()` if path does not exist (success), or an error if path does exist.
pub fn path_exists<S>(path: S) -> Fallible<()>
where
    S: Into<String>,
{
    let path = path.into();
    let meta = std::fs::metadata(&path);

    // check if path already exists
    if meta.is_ok() {
        return Err(PassrsError::PathExists(path).into());
    }

    check_sneaky_paths(&path)?;

    Ok(())
}

// TODO: check for .. and shell expansion
fn check_sneaky_paths(path: &str) -> Fallible<()> {
    let _ = path;

    if false {
        return Err(PassrsError::SneakyPath(path.to_owned()).into());
    }

    Ok(())
}

/// Search in PASSWORD_STORE_DIR for `target`.
// TODO: fuzzy searching
// TODO: maybe frecency as well? (a la z, j, fasd, autojump, etc)
pub fn search_entries<S>(target: S) -> Fallible<Vec<String>>
where
    S: Into<String>,
{
    use walkdir::WalkDir;

    let target = target.into();
    let mut matches = Vec::new();

    for entry in WalkDir::new(&*PASSWORD_STORE_DIR) {
        let entry = entry?
            .into_path()
            .to_str()
            .ok_or_else(|| failure::err_msg("Path couldn't be converted to string"))?
            .to_string();

        if entry.contains(&target) && entry.ends_with(".gpg") {
            matches.push(entry);
        }
    }

    if !matches.is_empty() {
        Ok(matches)
    } else {
        Err(PassrsError::NoMatchesFound(target).into())
    }
}

/// Decrypts the file into a `Vec` of `String`s. This will return an `Err` if
/// the plaintext is not validly UTF8 encoded.
pub fn decrypt_file_into_vec<S>(file: S) -> Fallible<Vec<String>>
where
    S: Into<String>,
{
    let file = file.into();

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let mut cipher = Data::load(file)?;
    let mut plain = Vec::new();
    ctx.decrypt(&mut cipher, &mut plain)?;

    let plain = std::str::from_utf8(&plain)?;
    let out = plain.lines().map(ToOwned::to_owned).collect();

    Ok(out)
}

/// Decrypts the given file into a `Vec` of bytes.
pub fn decrypt_file_into_bytes<S>(file: S) -> Fallible<Vec<u8>>
where
    S: Into<String>,
{
    let file = file.into();

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let mut cipher = Data::load(file)?;
    let mut plain = Vec::new();
    ctx.decrypt(&mut cipher, &mut plain)?;

    Ok(plain)
}

/// Creates all directories to allow `file` to be created.
pub fn create_descending_dirs(file: &str) -> Fallible<()> {
    let sep = file.rfind('/').unwrap_or(0);
    let dir = &file[..sep];
    std::fs::create_dir_all(dir)?;

    Ok(())
}

/// Encrypts data (a slice of bytes) using PASSWORD_STORE_SIGNING_KEY,
/// PASSWORD_STORE_KEY, or the key listed in `.gpg-id`, as the fallback path.
/// Callers must verify that `PASSWORD_STORE_DIR` exists and is initialized
/// (usually by verifying the `.gpg-id` file exists and a `git` repo has been
/// initialized).
pub fn encrypt_bytes_into_file<S>(file: S, plain: &[u8]) -> Fallible<()>
where
    S: Into<String>,
{
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::io::Write;

    let file = file.into();
    create_descending_dirs(&file)?;

    dbg!(&file);
    let mut file = match OpenOptions::new().write(true).open(&file) {
        Ok(file) => file,
        Err(_) => OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file)?,
    };
    dbg!(&file);
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

    let mut gpg_id = String::new();
    let mut id_file = OpenOptions::new()
        .read(true)
        .open(format!("{}/.gpg-id", *PASSWORD_STORE_DIR))?;
    dbg!(&id_file);
    id_file.read_to_string(&mut gpg_id)?;

    let mut signing_key = None;

    for &key in [&*PASSWORD_STORE_SIGNING_KEY, &*PASSWORD_STORE_KEY, &gpg_id].iter() {
        match ctx.get_key(key) {
            Ok(key) => {
                signing_key = Some(key);
                break;
            }
            Err(_) => continue,
        };
    }

    if let Some(key) = signing_key {
        let mut cipher = Vec::new();
        ctx.encrypt(&[key], plain, &mut cipher)?;
        file.write_all(&cipher)?;
    } else {
        return Err(PassrsError::NoSigningKeyFound.into());
    }

    Ok(())
}

fn git_open<S>(path: S) -> Fallible<Repository>
where
    S: Into<String>,
{
    let path = path.into();

    check_sneaky_paths(&path)?;

    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("failed to open git repo: {:?}", e);
            return Err(PassrsError::FailedToOpenGitRepo.into());
        }
    };

    Ok(repo)
}

/// Commit everything using `commit_message` as the message, if the workspace is
/// dirty. Callers must verify that `PASSWORD_STORE_DIR` exists and is
/// initialized (usually by verifying the `.gpg-id` file exists and a `git` repo
/// has been initialized).
pub fn commit(commit_message: String) -> Fallible<()> {
    // TODO:
    // [master 5a5604a] Add generated password for test.
    //  1 file changed, 0 insertions(+), 0 deletions(-)
    //  create mode 100644 test.gpg
    let path = &*PASSWORD_STORE_DIR;

    if let Ok(repo) = git_open(path) {
        if repo.statuses(None)?.is_empty() {
            dbg!("Nothing to do");
            return Ok(());
        }

        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
        ctx.set_armor(true);

        // Get ready to commit
        let mut index = repo.index()?;
        index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let tree_id = repo.index()?.write_tree()?;
        let sig = repo.signature()?;
        let mut parents = Vec::new();

        if let Some(parent) = repo.head().ok().map(|h| h.target().unwrap()) {
            parents.push(repo.find_commit(parent)?);
        }

        let parents = parents.iter().collect::<Vec<_>>();
        let buf = repo.commit_create_buffer(
            &sig,
            &sig,
            &commit_message,
            &repo.find_tree(tree_id)?,
            &parents,
        )?;
        let contents = buf
            .as_str()
            .ok_or_else(|| err_msg("Buffer was not valid UTF-8"))?;
        let mut outbuf = Vec::new();

        ctx.sign(SignMode::Detached, contents, &mut outbuf)?;

        let out = std::str::from_utf8(&outbuf)?;
        let ret = repo.commit_signed(&contents, &out, Some("gpgsig"))?;

        // NOTE: also implemented here: subcmds/init.rs
        // TODO: Unless I force it, anything after the first commit won't
        // actually get committed
        repo.reference(
            "refs/heads/master",
            ret,
            true,
            &format!("commit: {}", commit_message),
        )?;

        dbg!(ret);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_tester() {
        let paths = [
            "Internet/amazon.com/password.gpg",
            "~/.password-store/Internet/amazon.com/password.gpg",
            &format!(
                "{}/.password-store/Internet/amazon.com/password.gpg",
                std::env::var("HOME").unwrap()
            ),
        ];

        for path in &paths {
            assert_eq!(
                canonicalize_path(path).unwrap(),
                format!(
                    "{}/.password-store/Internet/amazon.com/password.gpg",
                    std::env::var("HOME").unwrap()
                )
            );
        }
    }
}
