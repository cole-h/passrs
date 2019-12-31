use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::io::Write;

use std::path::{Path, PathBuf};

use failure::{err_msg, Fallible};
use git2::Repository;
use gpgme::{Context, Data, Protocol, SignMode};

use crate::consts::{PASSWORD_STORE_DIR, PASSWORD_STORE_SIGNING_KEY};
use crate::error::PassrsError;
use crate::util;

// TODO: The init command will keep signatures of .gpg-id files up to date.
// TODO: key can be a vec of keys
pub fn init(path: Option<String>, key: Option<String>) -> Fallible<()> {
    let key = key.unwrap_or_else(|| PASSWORD_STORE_SIGNING_KEY.to_owned());
    let store = PASSWORD_STORE_DIR.to_owned();

    // If store doesn't exist, create it
    if util::path_exists(&store).is_ok() {
        create_store(store, &key)?;
    } else {
        // TODO: remove me
        // #[cfg(debug_assertions)]
        // assert_eq!("", " ");

        if let Some(path) = path {
            let path = util::canonicalize_path(&path)?;

            // User specified a subpath, so create a substore at that path
            if util::path_exists(&path).is_ok() {
                // Path doesn't exist, so we can create it
                create_store(path, &key)?; // TODO: don't use create_store, because we don't want to init git (already has been)
                update_key(&store, &key)?;
            } else if !compare_keys(&path, &key)? {
                // Keys aren't the same, so we can recrypt this subdir

                // `recrypt_store` handles the case where a subdir has a .gpg-id
                // (which causes it to break out of the loop, thus ignoring any
                // dir with a .gpg-id except for the root, PASSWORD_STORE_DIR)
                // TODO: is there any way to NOT need a clone?
                recrypt_store(&store, vec![key.clone()])?;
                update_key(&store, &key)?;
            // TODO: need to commit this -- custom reflog message
            } else {
                // TODO: if key is given, recrypt that path
                // Path exists, error out
                return Err(PassrsError::PathExists(path).into());
            }
        } else if compare_keys(&store, &key)? {
            // If the keys are the same, the supplied key is the current key
            return Err(PassrsError::SameKey(key).into());
        } else {
            // `recrypt_store` handles the case where a subdir has a .gpg-id
            // (which causes it to break out of the loop, thus ignoring any
            // dir with a .gpg-id except for the root, PASSWORD_STORE_DIR)
            // TODO: is there any way to NOT need a clone?
            recrypt_store(&store, vec![key.clone()])?;
            update_key(&store, &key)?;
            // TODO: need to commit this -- custom reflog message
        }
    }

    Ok(())
}

fn recrypt_store<P: Into<PathBuf>>(dir: P, keys: Vec<String>) -> Fallible<()> {
    let dir = dir.into();

    // get directory's contents
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_str()
            .map(|e| !e.starts_with(".git"))
            .unwrap_or(false)
        {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if name == ".gpg-id" {
                    dbg!(&path);
                    if *path == PathBuf::from(format!("{}.gpg-id", *PASSWORD_STORE_DIR)) {
                        continue;
                    }
                    break;
                }
            }
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "gpg" {
                        recrypt_file(path, keys.clone())?;
                    }
                }
            } else if path.is_dir() {
                // Keep descending file tree
                recrypt_store(path, keys.clone())?;
            }
        }
    }

    Ok(())
}

fn recrypt_file<S>(file: S, keys: Vec<String>) -> Fallible<()>
where
    S: AsRef<Path>,
{
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let keys = keys
        .iter()
        .map(|k| ctx.get_key(k))
        .filter_map(|k| k.ok())
        .collect::<Vec<_>>();
    let mut cipher = Data::load(file.as_ref().to_str().unwrap())?;
    let mut plain = Vec::new();

    ctx.decrypt(&mut cipher, &mut plain)?;

    let mut cipher = Vec::new();
    let mut file = OpenOptions::new().write(true).open(file.as_ref())?;

    ctx.encrypt(keys.iter(), &plain, &mut cipher)?;
    file.write_all(&cipher)?;

    Ok(())
}

fn update_key(path: &str, key: &str) -> Fallible<()> {
    let gpg_id = verify_key(key)?;
    let gpg_id_path = format!("{}/.gpg-id", path);

    // create .gpg-id
    let mut file = File::create(&gpg_id_path)?;
    file.write_all(gpg_id.as_bytes())?;

    Ok(())
}

fn git_init(path: &str) -> Fallible<Repository> {
    let repo = Repository::init(path)?;
    Ok(repo)
}

// fn git_open(path: &str) -> Fallible<Repository> {
//     let repo = Repository::open(path)?;
//     Ok(repo)
// }

// TODO: like gopass, iterate over keys that match
fn verify_key<S>(gpg_key: S) -> Fallible<String>
where
    S: Into<String>,
{
    let key = gpg_key.into();
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

    if let Ok(secret_key) = ctx.get_secret_key(&key) {
        let user_id = if let Ok(email) = secret_key
            .user_ids()
            .nth(0)
            .ok_or_else(|| err_msg("Option did not contain a value."))?
            .email()
        {
            email.to_string()
        } else {
            key
        };

        Ok(user_id)
    } else {
        Err(PassrsError::NoPrivateKeyFound.into())
    }
}

fn compare_keys(path: &str, key: &str) -> Fallible<bool> {
    let store_key = format!("{}/.gpg-id", path);
    let mut keyfile = fs::OpenOptions::new().read(true).open(store_key)?;
    let mut id = String::new();
    keyfile.read_to_string(&mut id)?;
    let id = id.trim();

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let key1 = ctx.get_secret_key(id);
    let key2 = ctx.get_secret_key(key);

    match (&key1, &key2) {
        (Ok(key1), Ok(key2)) => {
            // unwrap_or to make sure that if they are both None, they are
            // different and thus don't falsely return true
            if key1.id().unwrap_or("1") == key2.id().unwrap_or("2") {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}

// TODO: abstract away so most of the innards can be used for setup_store
fn create_store(path: String, gpg_key: &String) -> Fallible<()> {
    let gpg_id = verify_key(gpg_key)?;

    fs::create_dir_all(&path)?;

    if let Ok(repo) = git_init(&path) {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

        // create .gpg-id
        let gpg_id_path = format!("{}/.gpg-id", path);
        let mut file = File::create(&gpg_id_path)?;
        file.write_all(gpg_id.as_bytes())?;

        // create pass .gitattributes
        let gitattributes_path = format!("{}/.gitattributes", path);
        let mut file = File::create(&gitattributes_path)?;
        file.write_all(b"*.gpg diff=gpg")?;

        // get ready to commit
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

        // NOTE: this creates a non-PGP-signed commit.
        // let ret = repo.commit(
        //     Some("HEAD"),
        //     &sig,
        //     &sig,
        //     &format!("Password store initialized for {}", gpg_id),
        //     &repo.find_tree(tree_id)?,
        //     &parents,
        // )?;

        let buf = repo.commit_create_buffer(
            &sig,
            &sig,
            &format!("Password store initialized for {}", gpg_id),
            &repo.find_tree(tree_id)?,
            &parents,
        )?;
        let contents = std::str::from_utf8(&buf)?.to_string();
        let mut outbuf = Vec::new();

        ctx.set_armor(true);
        ctx.sign(SignMode::Detached, &*buf, &mut outbuf)?;

        let out = std::str::from_utf8(&outbuf)?;
        let commit = repo.commit_signed(&contents, &out, Some("gpgsig"))?;

        // TODO: verify there are no side-effects to this
        // If you use "HEAD" as the ref to change, master isn't updated. Short
        // refs don't work.
        match repo.reference("refs/heads/master", commit, false, "TODO: init message") {
            Ok(reference) => reference,
            Err(_) => repo.reference("refs/heads/master", commit, true, "TODO: reinit message")?,
        };

        // TODO: remove
        dbg!(commit);
    }

    Ok(())
}

// glob for a list of dirs containing .gpg-id
// blacklist those dirs
// return early if they are encountered
// dbg!(glob::Pattern::new("/tmp/passrstest/**/.gpg-id")?);

// TODO: use in find for filtering the tree?
// let mut blacklist = vec![
//     PathBuf::from(format!("{}.git", &store)),
//     PathBuf::from(format!("{}.gpg-id", &store)),
// ];
// for entry in glob::glob("/tmp/passrstest/**/.gpg-id")? {
//     let entry = entry?;
//     let parent = entry.parent().unwrap().to_path_buf();
//     if parent != PathBuf::from(&store) {
//         blacklist.push(parent);
//     }
// }

// fn is_not_hidden(entry: &walkdir::DirEntry) -> bool {
//     entry
//         .file_name()
//         .to_str()
//         .map(|s| entry.depth() == 0 || !s.starts_with("."))
//         .unwrap_or(false)
// }

// for entry in walkdir::WalkDir::new(&store)
//     .min_depth(1)
//     .max_depth(1)
//     .into_iter()
//     .filter_entry(|e| is_not_hidden(e))
//     .filter_map(|v| v.ok())
// {
//     // let entry = entry?;
//     if !blacklist.iter().any(|p| entry.path() == p) {
//         // targets.push(entry.into_path());
//         if entry.path().is_dir() {
//             //
//         }
//     }
// }
