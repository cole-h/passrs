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
    if !util::path_exists(&store)? {
        if let Some(path) = path {
            println!("Ignoring path {}; creating store at {}", path, &store);
        }
        create_store(store, &key)?;
    } else {
        // Store does exist
        if let Some(path) = path {
            // User specified a subpath, so create a substore at that path
            let substore_path = util::exact_path(&path)?;

            if !util::path_exists(&substore_path)? {
                // Substore doesn't exist yet, so we can create it
                create_substore(&store, &substore_path, &key)?;
                println!("Password store initialized for {} ({})", &key, &path);
            } else if compare_keys(&substore_path, &key)? {
                // Substore exists at `substore_path` and keys are the same
                return Err(PassrsError::SameKey(key).into());
            } else {
                // Substore exists at `substore_path` and keys aren't the same
                // so we can recrypt this subdir

                // `recrypt_store` handles the case where a subdir has a .gpg-id
                // (which causes it to break out of the loop, thus ignoring any
                // dir with a .gpg-id except for the root, PASSWORD_STORE_DIR)
                // TODO: is there any way to NOT need a clone?
                // TODO: if key is given, recrypt that path
                // Path exists, error out
                recrypt_store(&substore_path, [key.clone()])?;
                update_key(&substore_path, &key)?;
                // need to commit everything -- just use util::commit
                util::commit(format!(
                    "Re-encrypt {} using new GPG ID {}",
                    &substore_path.display().to_string()[PASSWORD_STORE_DIR.len()..],
                    &key
                ))?;
                // return Err(PassrsError::PathExists(substore_path.display().to_string()).into());
            }
        } else if compare_keys(&store, &key)? {
            // If the keys are the same, the supplied key is the current key
            return Err(PassrsError::SameKey(key).into());
        } else {
            // `recrypt_store` handles the case where a subdir has a .gpg-id
            // (which causes it to break out of the loop, thus ignoring any
            // dir with a .gpg-id except for the root, PASSWORD_STORE_DIR)
            // TODO: is there any way to NOT need a clone?
            recrypt_store(&store, [key.clone()])?;
            update_key(&store, &key)?;
            util::commit(format!(
                "Re-encrypt password store using new GPG ID {}",
                &key
            ))?;
        }
    }

    Ok(())
}

fn recrypt_store<P, V>(dir: P, keys: V) -> Fallible<()>
where
    P: AsRef<Path>,
    V: AsRef<[String]>,
{
    let keys = keys.as_ref();
    let dir = dir.as_ref();

    // get directory's contents
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name();
        let show = entry
            .file_name()
            .to_str()
            .map(|e| !e.starts_with(".git"))
            .unwrap_or(false);
        if show {
            if let Some(name) = file_name {
                dbg!(&path);
                if name == ".gpg-id" {
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

fn recrypt_file<S, V>(file: S, keys: V) -> Fallible<()>
where
    S: AsRef<Path>,
    V: AsRef<[String]>,
{
    let keys = keys.as_ref();

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

fn update_key<S, P>(path: P, key: S) -> Fallible<()>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let path = path.as_ref();
    let key = key.as_ref();

    let gpg_id = verify_key(key)?;
    let gpg_id_path = format!("{}/.gpg-id", path.display());

    // create .gpg-id
    let mut file = File::create(&gpg_id_path)?;
    file.write_all(gpg_id.as_bytes())?;

    Ok(())
}

fn verify_key<S>(gpg_key: S) -> Fallible<String>
where
    S: Into<String>,
    // TODO: like gopass, iterate over keys that match
    // S: AsRef<[String]>,
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
            email.to_owned()
        } else {
            key
        };

        Ok(user_id)
    } else {
        Err(PassrsError::NoPrivateKeyFound.into())
    }
}

fn compare_keys<P>(path: P, key: &str) -> Fallible<bool>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let store_key = format!("{}/.gpg-id", path.display());
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

fn git_prep(repo: &Repository) -> Fallible<(git2::Oid, git2::Signature, Vec<git2::Commit>)> {
    let mut index = repo.index()?;

    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_id = repo.index()?.write_tree()?;
    let sig = repo.signature()?;
    let mut parents = Vec::new();

    if let Some(parent) = repo.head().ok().map(|h| h.target().unwrap()) {
        parents.push(repo.find_commit(parent)?);
    }

    let parents = parents.iter().map(ToOwned::to_owned).collect::<Vec<_>>();

    // NOTE: this creates a non-PGP-signed commit.
    // let ret = repo.commit(
    //     Some("HEAD"),
    //     &sig,
    //     &sig,
    //     &format!("Password store initialized for {}", gpg_id),
    //     &repo.find_tree(tree_id)?,
    //     &parents,
    // )?;

    Ok((tree_id, sig, parents))
}

// TODO: abstract away so most of the innards can be used for setup_store
fn create_store<P, S>(path: P, gpg_key: S) -> Fallible<()>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let path = path.as_ref();
    let gpg_key = gpg_key.as_ref();
    let gpg_id = verify_key(gpg_key)?;

    match fs::metadata(&path) {
        Ok(_) => {}
        Err(_) => fs::create_dir_all(&path)?,
    }
    // fs::create_dir_all(&path)?;

    if let Ok(repo) = Repository::init(&path) {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

        // create .gpg-id
        let gpg_id_path = format!("{}/.gpg-id", path.display());
        let mut file = File::create(&gpg_id_path)?;
        file.write_all(gpg_id.as_bytes())?;

        // create pass .gitattributes
        let gitattributes_path = format!("{}/.gitattributes", path.display());
        let mut file = File::create(&gitattributes_path)?;
        file.write_all(b"*.gpg diff=gpg")?;

        let (tree_id, sig, parents) = git_prep(&repo)?;
        let parents = parents
            .iter()
            .map(std::borrow::Borrow::borrow)
            .collect::<Vec<_>>();

        // get ready to commit
        let buf = repo.commit_create_buffer(
            &sig,
            &sig,
            &format!("Password store initialized for {}", gpg_key),
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
        repo.reference("refs/heads/master", commit, true, "TODO: init message")?;

        // TODO: remove
        dbg!(commit);
    }

    Ok(())
}

// TODO: abstract away so most of the innards can be used for setup_store
fn create_substore<P, Q, S>(store: P, path: Q, gpg_key: S) -> Fallible<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    S: AsRef<str>,
{
    let path = path.as_ref();
    let store = store.as_ref();
    let gpg_key = gpg_key.as_ref();
    let _ = verify_key(gpg_key)?;

    match fs::metadata(&path) {
        Ok(_) => {}
        Err(_) => fs::create_dir_all(&path)?,
    }
    // fs::create_dir_all(&path)?;

    if let Ok(repo) = Repository::open(&store) {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

        // create .gpg-id
        let gpg_id_path = format!("{}/.gpg-id", path.display());
        let mut file = File::create(&gpg_id_path)?;
        file.write_all(gpg_key.as_bytes())?;

        let (tree_id, sig, parents) = git_prep(&repo)?;
        let parents = parents
            .iter()
            .map(std::borrow::Borrow::borrow)
            .collect::<Vec<_>>();

        // get ready to commit
        let buf = repo.commit_create_buffer(
            &sig,
            &sig,
            &format!(
                "Set GPG ID to {} ({})",
                gpg_key,
                &path.display().to_string()[PASSWORD_STORE_DIR.len()..]
            ),
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
        repo.reference("refs/heads/master", commit, true, "TODO: init message")?;

        // TODO: remove
        dbg!(commit);
    }

    Ok(())
}

// TODO: clean up these notes
//     used to use glob/ignore, but no longer necessary because of my kludgy
//     function, `recrypt_store`
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
