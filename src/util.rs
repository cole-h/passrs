use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::str;

use anyhow::{Context as _, Result};
use git2::Repository;
use gpgme::{Context, Data, Protocol, SignMode};
use rand::Rng;
use termion::input::TermRead;
use walkdir::WalkDir;

use crate::consts::{
    GPG_ID_FILE, HOME, PASSWORD_STORE_DIR, PASSWORD_STORE_KEY, PASSWORD_STORE_SIGNING_KEY,
};
use crate::PassrsError;

// TODO: check paths in every function that reads or writes to .password-store
// TODO: move all git-related fns to a git.rs
// TODO: all functions except init require that the password store exists
// (git repo must be init'd, and .gpg-id must be present)

// TODO: for all commands that write to the store, create_descending_dirs
// 1. canonicalize_path
// 2. path_exists (which implicitly check_sneaky_paths)
// 3. sep=rfind('/') and fs::create_all_dirs(path[..sep])

// TODO: verify function doesn't munge the path
/// Paths may be an absolute path to the entry, or relative to the store's root.
pub fn canonicalize_path<S>(path: S) -> Result<PathBuf>
where
    S: AsRef<str>,
{
    let path = path.as_ref();
    let mut path = path.replace("~", &HOME);

    if !path.contains(&*PASSWORD_STORE_DIR) {
        path = [PASSWORD_STORE_DIR.to_owned(), path].concat();
    }

    self::check_sneaky_paths(&path)?;

    path = match fs::metadata(&path) {
        Ok(_) => path,
        Err(_) => {
            if !path.ends_with(".gpg") {
                path + ".gpg"
            } else {
                path
            }
        }
    };

    Ok(PathBuf::from(path))
}

pub fn exact_path<S>(path: S) -> Result<PathBuf>
where
    S: AsRef<str>,
{
    let path = path.as_ref();
    let mut path = path.replace("~", &HOME);

    if !path.contains(&*PASSWORD_STORE_DIR) {
        path = [PASSWORD_STORE_DIR.to_owned(), path].concat();
    }

    self::check_sneaky_paths(&path)?;

    // TODO: callers should create_descending_dirs when appropriate
    // create_descending_dirs(&path)?;

    Ok(PathBuf::from(path))
}

pub fn verify_store_exists() -> Result<()> {
    let meta = fs::metadata(&*PASSWORD_STORE_DIR);
    if meta.is_err() {
        return Err(PassrsError::StoreDoesntExist.into());
    }

    let gpg_id = &*GPG_ID_FILE;
    let meta = fs::metadata(gpg_id);
    if meta.is_err() {
        return Err(PassrsError::StoreDoesntExist.into());
    }

    Ok(())
}

/// Returns `false` if path does not exist (success), `true` if it does exist.
pub fn path_exists<P>(path: P) -> Result<bool>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let meta = fs::metadata(&path);

    // check if path already exists
    if meta.is_ok() {
        return Ok(true);
    }

    self::check_sneaky_paths(path)?;

    Ok(false)
}

// TODO: check for .. and shell expansion
// TODO: only allowed to specify in PASSWORD_STORE_DIR
fn check_sneaky_paths<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let strpath = path.to_str().unwrap();

    if strpath.contains("../") || strpath.contains("/..") {
        return Err(PassrsError::SneakyPath(path.display().to_string()).into());
    }
    if !strpath.contains(&*PASSWORD_STORE_DIR) {
        return Err(PassrsError::SneakyPath(path.display().to_string()).into());
    }

    Ok(())
}

/// Search in PASSWORD_STORE_DIR for `target`.
// TODO: fuzzy searching
// TODO: maybe frecency as well? (a la z, j, fasd, autojump, etc)
pub fn find_target_single<S>(target: S) -> Result<Vec<String>>
where
    S: AsRef<str> + ToString,
{
    let mut matches: Vec<String> = Vec::new();
    let target = target.as_ref();

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
        let filename = &entry.file_name().to_str().unwrap();
        let path = entry.clone().into_path().to_str().unwrap().to_owned();

        if filename == &target.to_owned() && path.ends_with(".gpg") {
            return Ok(vec![path]);
        }

        if path[PASSWORD_STORE_DIR.len()..].contains(target) && is_file && path.ends_with(".gpg") {
            matches.push(path.to_owned());
        }
    }

    if matches.is_empty() {
        Err(PassrsError::NoMatchesFound(target.to_owned()).into())
    } else {
        Ok(matches)
    }
}

// TODO: fuzzy search feature -- 5 closest matches
// pub fn find_target_multi<V>(targets: V) -> Result<Vec<String>>
// where
//     V: AsRef<[String]>,
// {
//     let targets = targets.as_ref();
//     let mut matches = Vec::new();

//     for target in targets {
//         for entry in WalkDir::new(&*PASSWORD_STORE_DIR) {
//             let entry = entry?.into_path().to_str().unwrap().to_owned();

//             if entry[PASSWORD_STORE_DIR.len()..].contains(target) {
//                 matches.push(entry);
//             }
//         }
//     }
//     if matches.is_empty() {
//         Err(PassrsError::NoMatchesFoundMultiple(targets.to_vec()).into())
//     } else {
//         Ok(matches)
//     }
// }

/// Decrypts the file into a `Vec` of `String`s. This will return an `Err` if
/// the plaintext is not validly UTF8 encoded.
pub fn decrypt_file_into_strings<S>(file: S) -> Result<Vec<String>>
where
    S: AsRef<str>,
{
    let file = file.as_ref();

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let mut cipher = Data::load(file)?;
    let mut plain = Vec::new();
    ctx.decrypt(&mut cipher, &mut plain)?;

    let plain = str::from_utf8(&plain)?;
    let out = plain.lines().map(ToOwned::to_owned).collect();

    Ok(out)
}

/// Decrypts the given file into a `Vec` of bytes.
pub fn decrypt_file_into_bytes<S>(file: S) -> Result<Vec<u8>>
where
    S: AsRef<Path>,
{
    let file = file.as_ref();
    let mut bytes = Vec::new();
    let mut file = fs::File::open(file)?;
    file.read_to_end(&mut bytes)?;

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let mut cipher = Data::from_bytes(bytes)?;
    let mut plain = Vec::new();
    ctx.decrypt(&mut cipher, &mut plain)?;

    Ok(plain)
}

/// Creates all directories to allow `file` to be created.
pub fn create_descending_dirs<S>(file: S) -> Result<()>
where
    S: AsRef<Path>,
{
    let file = file.as_ref();
    let file = file.display().to_string();
    let sep = file
        .rfind('/')
        .with_context(|| format!("No folder found in file {}", file))?;
    let dir = &file[..sep];
    fs::create_dir_all(dir)?;

    Ok(())
}

/// Encrypts data (a slice of bytes) using PASSWORD_STORE_SIGNING_KEY,
/// PASSWORD_STORE_KEY, or the key listed in `.gpg-id`, as the fallback path.
/// Callers must verify that `PASSWORD_STORE_DIR` exists and is initialized
/// (usually by verifying the `.gpg-id` file exists and a `git` repo has been
/// initialized).
pub fn encrypt_bytes_into_file<S>(plain: &[u8], path: S) -> Result<()>
where
    S: AsRef<Path>,
{
    let path = path.as_ref();
    self::create_descending_dirs(&path)?;

    dbg!(&path);
    let mut file = fs::OpenOptions::new()
        .mode(0o600)
        .write(true)
        .create(true)
        .open(&path)?;
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let id_file = fs::OpenOptions::new().read(true).open(&*GPG_ID_FILE)?;
    let reader = BufReader::new(&id_file);
    let mut keys = vec![
        PASSWORD_STORE_SIGNING_KEY.to_string(),
        PASSWORD_STORE_KEY.to_string(),
    ];

    for line in reader.lines() {
        let line = line?;
        keys.push(line);
    }

    let mut signing_keys = Vec::new();

    for key in keys.iter() {
        match ctx.get_key(key) {
            Ok(key) => signing_keys.push(key),
            Err(_) => continue,
        };
    }

    if signing_keys.is_empty() {
        return Err(PassrsError::NoSigningKeyFound.into());
    } else {
        let mut cipher = Vec::new();

        ctx.encrypt(&signing_keys, plain, &mut cipher)?;
        file.write_all(&cipher)?;
    }

    Ok(())
}

pub fn append_encrypted_bytes<S>(plain: &[u8], path: S) -> Result<()>
where
    S: AsRef<Path>,
{
    let path = path.as_ref();
    create_descending_dirs(&path)?;

    dbg!(&path);
    let mut file = fs::OpenOptions::new()
        .mode(0o600)
        .write(true)
        .create(true)
        .open(&path)?;
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let id_file = fs::OpenOptions::new().read(true).open(&*GPG_ID_FILE)?;
    let reader = BufReader::new(&id_file);
    let mut keys = vec![
        PASSWORD_STORE_SIGNING_KEY.to_string(),
        PASSWORD_STORE_KEY.to_string(),
    ];

    for line in reader.lines() {
        let line = line?;
        keys.push(line);
    }

    let mut signing_keys = Vec::new();

    for key in keys.iter() {
        match ctx.get_key(key) {
            Ok(key) => signing_keys.push(key),
            Err(_) => continue,
        };
    }
    dbg!(&signing_keys.len());

    if signing_keys.is_empty() {
        return Err(PassrsError::NoSigningKeyFound.into());
    } else {
        let mut input = Data::load(path.display().to_string())?;
        let mut output = Vec::new();

        ctx.decrypt(&mut input, &mut output)?;
        output.push(b'\n');
        output.extend(plain);

        let mut contents = Vec::new();

        ctx.encrypt(&signing_keys, &mut output, &mut contents)?;
        file.write_all(&contents)?;
    }

    Ok(())
}

fn git_open<S>(path: S) -> Result<Repository>
where
    S: AsRef<str>,
{
    let path = path.as_ref();

    self::check_sneaky_paths(&path)?;

    let repo = Repository::open(path)?;

    Ok(repo)
}

/// Commit everything using `commit_message` as the message, if the workspace is
/// dirty. Callers must verify that `PASSWORD_STORE_DIR` exists and is
/// initialized (usually by verifying the `.gpg-id` file exists and a `git` repo
/// has been initialized).
pub fn commit<S>(commit_message: S) -> Result<()>
where
    S: AsRef<str>,
{
    // TODO:
    // [master 5a5604a] Add generated password for test.
    //  1 file changed, 0 insertions(+), 0 deletions(-)
    //  create mode 100644 test.gpg
    let path = &*PASSWORD_STORE_DIR;
    let commit_message = commit_message.as_ref();

    if let Ok(repo) = git_open(path) {
        if repo.statuses(None)?.is_empty() {
            dbg!("Nothing to do");
            return Ok(());
        }

        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

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
        let mut outbuf = Vec::new();

        ctx.set_armor(true);
        ctx.sign(SignMode::Detached, &*buf, &mut outbuf)?;

        let contents = buf.as_str().with_context(|| "Buffer was not valid UTF-8")?;
        let out = str::from_utf8(&outbuf)?;
        let commit = repo.commit_signed(&contents, &out, Some("gpgsig"))?;

        // NOTE: also implemented here: subcmds/init.rs
        repo.reference(
            "refs/heads/master",
            commit,
            true, // force-update the master HEAD, otherwise the commit will not be part of the tree
            &format!("commit: {}", commit_message),
        )?;

        // TODO: remove
        dbg!(commit);
    }

    Ok(())
}

fn uid(path: &Path) -> Result<(u64, u64)> {
    let metadata = path.metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

// TODO: investigate licensing; taken from
// --> https://github.com/mdunsmuir/copy_dir/blob/master/src/lib.rs#L118
pub fn copy<P, Q>(source: &P, dest: &Q, mut root: Option<(u64, u64)>) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let source = source.as_ref();
    let dest = dest.as_ref();
    let id = self::uid(source)?;
    let meta = source.metadata()?;

    // /home/vin/.password-store/Uncategorized/rclone
    // mkdir: created directory '/home/vin/.password-store/rclone'
    // '/home/vin/.password-store/Uncategorized/rclone' -> '/home/vin/.password-store/rclone/lmbo'
    // '/home/vin/.password-store/Uncategorized/rclone/password.gpg' -> '/home/vin/.password-store/rclone/lmbo/password.gpg'

    if meta.is_file() {
        match fs::metadata(&dest) {
            Ok(_) => {}
            Err(_) => self::create_descending_dirs(&dest)?,
        }

        // println!("'{}' -> '{}'", &source.display(), &dest.display());
        fs::copy(source, dest)?;
    } else if meta.is_dir() {
        if root.is_some() && root.unwrap() == id {
            return Err(PassrsError::SourceIsDestination.into());
        }

        fs::create_dir_all(dest)?;
        // println!("created directory {}", &dest.display());

        if root.is_none() {
            root = Some(self::uid(&dest)?);
        }

        // println!("'{}' -> '{}'", &source.display(), &dest.display());
        for entry in fs::read_dir(source)? {
            let entry = entry?.path();
            let name = entry.file_name().unwrap();
            self::copy(&entry, &dest.join(name), root)?;
        }

        fs::set_permissions(dest, meta.permissions())?;
    } else {
        // not file or dir
    }

    Ok(())
}

pub fn generate_chars_from_set<V>(set: V, len: usize) -> Result<Vec<u8>>
where
    V: AsRef<[u8]>,
{
    let set = set.as_ref();
    let mut rng = rand::thread_rng();

    let mut password_bytes = Vec::with_capacity(len);

    for _ in 0..len {
        let idx = rng.gen_range(0, set.len());
        password_bytes.push(set[idx]);
    }

    Ok(password_bytes)
}

pub fn prompt_for_secret<S>(echo: bool, secret_name: S) -> Result<Option<String>>
where
    S: AsRef<str>,
{
    let secret_name = secret_name.as_ref();

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let secret = if echo {
        write!(stdout, "Enter secret for {}: ", secret_name)?;
        io::stdout().flush()?;
        let input = TermRead::read_line(&mut stdin)?;

        if input.is_none() {
            return Err(PassrsError::UserAbort.into());
        }

        input
    } else {
        write!(stdout, "Enter secret for {}: ", secret_name)?;
        io::stdout().flush()?;
        let input = {
            let input = stdin.read_passwd(&mut stdout)?;
            writeln!(stdout)?;
            if input.is_none() {
                return Err(PassrsError::UserAbort.into());
            }

            input.unwrap()
        };

        write!(stdout, "Re-enter secret for {}: ", secret_name)?;
        io::stdout().flush()?;
        let check = {
            let input = stdin.read_passwd(&mut stdout)?;
            writeln!(stdout)?;
            if input.is_none() {
                return Err(PassrsError::UserAbort.into());
            }

            input.unwrap()
        };

        if input == check {
            Some(input)
        } else {
            return Err(PassrsError::SecretsDontMatch.into());
        }
    };

    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn canonicalize_tester() {
        let paths = [
            "Internet/amazon.com/password.gpg",
            "~/.password-store/Internet/amazon.com/password.gpg",
            &format!(
                "{}/.password-store/Internet/amazon.com/password.gpg",
                env::var("HOME").unwrap()
            ),
        ];

        for path in &paths {
            assert_eq!(
                canonicalize_path(path).unwrap().display().to_string(),
                format!(
                    "{}/.password-store/Internet/amazon.com/password.gpg",
                    env::var("HOME").unwrap()
                )
            );
        }
    }
}
