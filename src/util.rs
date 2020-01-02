use std::fs;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use failure::{err_msg, Fallible};
use git2::Repository;
use gpgme::{Context, Data, Protocol, SignMode};
use rand::Rng;
use walkdir::WalkDir;

use crate::consts::{HOME, PASSWORD_STORE_DIR, PASSWORD_STORE_KEY, PASSWORD_STORE_SIGNING_KEY};
use crate::error::PassrsError;

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
pub fn canonicalize_path<S>(path: S) -> Fallible<PathBuf>
where
    S: AsRef<str>,
{
    let path = path.as_ref();
    let mut path = path.replace("~", &*HOME);

    if !path.contains(&*PASSWORD_STORE_DIR) {
        path = [&*PASSWORD_STORE_DIR.to_owned(), &path].concat();
    }

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
    dbg!(&path);

    check_sneaky_paths(&path)?;
    // TODO: callers should create_descending_dirs when appropriate
    // create_descending_dirs(&path)?;

    Ok(PathBuf::from(path))
}

pub fn exact_path<S>(path: S) -> Fallible<PathBuf>
where
    S: AsRef<str>,
{
    let path = path.as_ref();
    let mut path = path.replace("~", &*HOME);

    if !path.contains(&*PASSWORD_STORE_DIR) {
        path = [&*PASSWORD_STORE_DIR.to_owned(), &path].concat();
    }

    check_sneaky_paths(&path)?;
    // TODO: callers should create_descending_dirs when appropriate
    // create_descending_dirs(&path)?;

    Ok(PathBuf::from(path))
}

pub fn verify_store_exists() -> Fallible<()> {
    let meta = fs::metadata(&*PASSWORD_STORE_DIR);
    if meta.is_err() {
        return Err(PassrsError::StoreDoesntExist.into());
    }

    let gpg_id = [&*PASSWORD_STORE_DIR, ".gpg-id"].concat();
    let meta = fs::metadata(gpg_id);
    if meta.is_err() {
        return Err(PassrsError::StoreDoesntExist.into());
    }

    Ok(())
}

/// Returns `false` if path does not exist (success), `true` if it does exist.
pub fn path_exists<P>(path: P) -> Fallible<bool>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let meta = fs::metadata(&path);

    // check if path already exists
    if meta.is_ok() {
        return Ok(true);
    }

    check_sneaky_paths(path)?;

    Ok(false)
}

// TODO: check for .. and shell expansion
// TODO: only allowed to specify in PASSWORD_STORE_DIR
fn check_sneaky_paths<P>(path: P) -> Fallible<()>
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
pub fn find_target_single<S>(target: S) -> Fallible<Vec<String>>
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

        if filename == &target.to_owned() {
            return Ok(vec![path]);
        }

        if path[PASSWORD_STORE_DIR.len()..].contains(target) && is_file {
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
pub fn find_target_multi<V>(targets: V) -> Fallible<Vec<String>>
where
    V: AsRef<[String]>,
{
    let targets = targets.as_ref();
    let mut matches = Vec::new();

    for target in targets {
        for entry in WalkDir::new(&*PASSWORD_STORE_DIR) {
            let entry = entry?.into_path().to_str().unwrap().to_owned();

            if entry[PASSWORD_STORE_DIR.len()..].contains(target) {
                matches.push(entry);
            }
        }
    }
    if matches.is_empty() {
        Err(PassrsError::NoMatchesFoundMultiple(targets.to_vec()).into())
    } else {
        Ok(matches)
    }
}

/// Decrypts the file into a `Vec` of `String`s. This will return an `Err` if
/// the plaintext is not validly UTF8 encoded.
pub fn decrypt_file_into_strings<S>(file: S) -> Fallible<Vec<String>>
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
pub fn create_descending_dirs<S>(file: S) -> Fallible<()>
where
    S: AsRef<Path>,
{
    let file = file.as_ref();
    let file = file.display().to_string();
    let sep = file.rfind('/').unwrap_or(0);
    let dir = &file[..sep];
    fs::create_dir_all(dir)?;

    Ok(())
}

/// Encrypts data (a slice of bytes) using PASSWORD_STORE_SIGNING_KEY,
/// PASSWORD_STORE_KEY, or the key listed in `.gpg-id`, as the fallback path.
/// Callers must verify that `PASSWORD_STORE_DIR` exists and is initialized
/// (usually by verifying the `.gpg-id` file exists and a `git` repo has been
/// initialized).
pub fn encrypt_bytes_into_file<S>(plain: &[u8], file: S) -> Fallible<()>
where
    S: AsRef<Path>,
{
    let file = file.as_ref();
    create_descending_dirs(&file)?;

    dbg!(&file);
    let mut file = match fs::OpenOptions::new().write(true).open(&file) {
        Ok(file) => file,
        Err(_) => fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file)?,
    };
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

    let mut gpg_id = String::new();
    let mut id_file = fs::OpenOptions::new()
        .read(true)
        .open(format!("{}/.gpg-id", *PASSWORD_STORE_DIR))?;
    id_file.read_to_string(&mut gpg_id)?;

    let mut signing_key = None;

    for &key in [&*PASSWORD_STORE_SIGNING_KEY, &gpg_id, &*PASSWORD_STORE_KEY].iter() {
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
pub fn commit<S>(commit_message: S) -> Fallible<()>
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

        let contents = buf
            .as_str()
            .ok_or_else(|| err_msg("Buffer was not valid UTF-8"))?;
        let out = std::str::from_utf8(&outbuf)?;
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

fn uid(path: &Path) -> Fallible<(u64, u64)> {
    let metadata = path.metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

// TODO: investigate licensing; taken from
// --> https://github.com/mdunsmuir/copy_dir/blob/master/src/lib.rs#L118
pub fn copy<P, Q>(source: &P, dest: &Q, mut root: Option<(u64, u64)>) -> Fallible<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let source = source.as_ref();
    let id = uid(source)?;
    let meta = source.metadata()?;

    if meta.is_file() {
        fs::copy(source, dest)?;
    } else if meta.is_dir() {
        if root.is_some() && root.unwrap() == id {
            return Err(PassrsError::SourceIsDestination.into());
        }

        fs::create_dir_all(dest)?;

        if root.is_none() {
            root = Some(uid(&dest.as_ref())?);
        }

        for entry in fs::read_dir(source)? {
            let entry = entry?.path();
            let name = entry.file_name().unwrap();
            copy(&entry, &dest.as_ref().join(name), root)?;
        }

        fs::set_permissions(dest, meta.permissions())?;
    } else {
        // unknown type
    }

    Ok(())
}

pub fn generate_chars_from_set<V>(set: V, len: usize) -> Fallible<Vec<u8>>
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

// TODO: research licensing
// re: https://github.com/mdunsmuir/copy_dir/blob/0.1.2/src/lib.rs#L67
// fn copy_dir<Q: AsRef<Path>, P: AsRef<Path>>(from: P, to: Q) -> Fallible<()> {
//     if !from.as_ref().exists() {
//         return Err(std::io::Error::new(
//             std::io::ErrorKind::NotFound,
//             "source path does not exist",
//         )
//         .into());
//     } else if to.as_ref().exists() {
//         return Err(
//             std::io::Error::new(std::io::ErrorKind::AlreadyExists, "target path exists").into(),
//         );
//     }

//     if from.as_ref().is_file() {
//         fs::copy(&from, &to)?;
//     }

//     // Allows us to make this generic over files AND dirs
//     if !from.as_ref().is_file() {
//         fs::create_dir(&to)?;
//     }

//     let target_is_under_source = from
//         .as_ref()
//         .canonicalize()
//         .and_then(|fc| to.as_ref().canonicalize().map(|tc| (fc, tc)))
//         .map(|(fc, tc)| tc.starts_with(fc))?;

//     if target_is_under_source {
//         fs::remove_dir(&to)?;

//         return Err(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             "cannot copy to a path prefixed by the source path",
//         )
//         .into());
//     }

//     for entry in WalkDir::new(&from)
//         .min_depth(1)
//         .into_iter()
//         .filter_map(|e| e.ok())
//     {
//         let relative_path = match entry.path().strip_prefix(&from) {
//             Ok(rp) => rp,
//             Err(_) => panic!("strip_prefix failed; this is a probably a bug in copy_dir"),
//         };

//         let target_path = {
//             let mut target_path = to.as_ref().to_path_buf();
//             target_path.push(relative_path);
//             target_path
//         };

//         let source_metadata = match entry.metadata() {
//             Err(_) => continue,
//             Ok(md) => md,
//         };

//         if source_metadata.is_dir() {
//             fs::create_dir(&target_path)?;
//             fs::set_permissions(&target_path, source_metadata.permissions())?;
//         } else {
//             fs::copy(entry.path(), &target_path)?;
//         }
//     }

//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn canonicalize_tester() {
//         let paths = [
//             "Internet/amazon.com/password.gpg",
//             "~/.password-store/Internet/amazon.com/password.gpg",
//             &format!(
//                 "{}/.password-store/Internet/amazon.com/password.gpg",
//                 std::env::var("HOME").unwrap()
//             ),
//         ];

//         for path in &paths {
//             assert_eq!(
//                 canonicalize_path(path).unwrap().display().to_owned(),
//                 format!(
//                     "{}/.password-store/Internet/amazon.com/password.gpg",
//                     std::env::var("HOME").unwrap()
//                 )
//             );
//         }
//     }
// }
