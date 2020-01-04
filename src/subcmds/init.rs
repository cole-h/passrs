use std::borrow::Borrow;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::str;

use anyhow::{Context as _, Result};
use git2::Repository;
use gpgme::{Context, Data, Protocol, SignMode};

use crate::consts::{PASSWORD_STORE_DIR, PASSWORD_STORE_SIGNING_KEY};
use crate::util;
use crate::PassrsError;

pub fn init(path: Option<String>, keys: Vec<String>) -> Result<()> {
    let keys = if keys.is_empty() {
        vec![PASSWORD_STORE_SIGNING_KEY.to_owned()]
    } else {
        keys
    };
    let store = PASSWORD_STORE_DIR.to_owned();

    // If store doesn't exist, create it
    if !util::path_exists(&store)? {
        if let Some(path) = path {
            println!("Ignoring path {}; creating store at {}", path, &store);
        }

        create_store(store, &keys)?;
    } else {
        // No signing key given
        // TODO: No ID means deinitialize
        //   1. Remove .gpg-id, `git rm -qr .gpg-id`
        //   2. commit "Deinitialize {gpg_id}"
        if keys.is_empty() || keys.get(0).map(|k| k.is_empty()).unwrap_or(false) {
            return Err(PassrsError::NoPrivateKeyFound.into());
        }
        // Store does exist
        if let Some(path) = path {
            // User specified a subpath, so create a substore at that path
            let substore_path = util::exact_path(&path)?;

            if !util::path_exists(&substore_path)? {
                // Substore doesn't exist yet, so we can create it
                create_substore(&store, &substore_path, &keys)?;

                let list = &keys.join(", ");
                let keys = if keys.len() > 1 { &list } else { &keys[0] };
                println!("Password store initialized for {} ({})", &keys, &path);
            } else {
                // Substore exists at `substore_path` and keys aren't the same
                // so we can recrypt this subdir

                // `recrypt_store` handles the case where a subdir has a .gpg-id
                // (which causes it to break out of the loop, thus ignoring any
                // dir with a .gpg-id except for the root, PASSWORD_STORE_DIR)
                recrypt_store(&substore_path, &keys)?;

                let list = &keys.join(", ");
                let new_keys = if keys.len() > 1 { &list } else { &keys[0] };
                // need to commit everything -- just use util::commit
                util::commit(format!(
                    "Re-encrypt {} using new GPG ID {}",
                    &substore_path.display().to_string()[PASSWORD_STORE_DIR.len()..],
                    &new_keys,
                ))?;

                update_key(&substore_path, &keys)?;
            }
        } else {
            // `recrypt_store` handles the case where a subdir has a .gpg-id
            // (which causes it to break out of the loop, thus ignoring any
            // dir with a .gpg-id except for the root, PASSWORD_STORE_DIR)
            recrypt_store(&store, &keys)?;

            let list = &keys.join(", ");
            let new_keys = if keys.len() > 1 { &list } else { &keys[0] };
            util::commit(format!(
                "Re-encrypt password store using new GPG ID {}",
                &new_keys
            ))?;

            update_key(&store, &keys)?;
        }
    }

    Ok(())
}

fn recrypt_store<P, V>(dir: P, keys: V) -> Result<()>
where
    P: AsRef<Path>,
    V: AsRef<[String]>,
{
    let keys = keys.as_ref();
    let dir = dir.as_ref();

    if keys.is_empty() || keys.get(0).map(|k| k.is_empty()).unwrap_or(false) {
        return Err(PassrsError::NoPrivateKeyFound.into());
    }

    // Get directory's contents
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
                        recrypt_file(path, keys)?;
                    }
                }
            } else if path.is_dir() {
                // Keep descending the file tree
                recrypt_store(path, keys)?;
            }
        }
    }

    Ok(())
}

fn recrypt_file<S, V>(file: S, keys: V) -> Result<()>
where
    S: AsRef<Path>,
    V: AsRef<[String]>,
{
    let keys = keys.as_ref();
    let file = file.as_ref();

    if keys.is_empty() || keys.get(0).map(|k| k.is_empty()).unwrap_or(false) {
        return Err(PassrsError::NoPrivateKeyFound.into());
    }

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let keys = keys
        .iter()
        .map(|k| ctx.get_key(k))
        .filter_map(|k| k.ok())
        .collect::<Vec<_>>();
    let mut cipher = Data::load(file.display().to_string())?;
    let mut plain = Vec::new();

    ctx.decrypt(&mut cipher, &mut plain)?;

    let mut cipher = Vec::new();
    let mut file = OpenOptions::new().mode(0o600).write(true).open(file)?;

    ctx.encrypt(&keys, &plain, &mut cipher)?;
    file.write_all(&cipher)?;

    Ok(())
}

fn update_key<S, P>(path: P, keys: S) -> Result<()>
where
    P: AsRef<Path>,
    S: AsRef<[String]>,
{
    let path = path.as_ref();
    let keys = keys.as_ref();

    if keys.is_empty() || keys.get(0).map(|k| k.is_empty()).unwrap_or(false) {
        return Err(PassrsError::NoPrivateKeyFound.into());
    }

    let gpg_ids = verify_keys(keys)?;
    let gpg_id_path = format!("{}.gpg-id", path.display());
    let mut file = fs::OpenOptions::new()
        .mode(0o600)
        .truncate(true)
        .write(true)
        .create(true)
        .open(&gpg_id_path)?;
    file.write_all(gpg_ids.join("\n").as_bytes())?;

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let gpg_id_signature = format!("{}.sig", gpg_id_path);
    let mut outbuf: Vec<u8> = Vec::new();
    let mut file = fs::OpenOptions::new()
        .mode(0o600)
        .truncate(true)
        .write(true)
        .create(true)
        .open(&gpg_id_signature)?;

    ctx.sign(
        SignMode::Detached,
        gpg_ids.join("\n").as_bytes(),
        &mut outbuf,
    )?;
    file.write_all(&outbuf)?;

    let list = &keys.join(", ");
    let new_keys = if keys.len() > 1 { &list } else { &keys[0] };

    util::commit(format!("Signing new GPG ID with {}", &new_keys))?;

    Ok(())
}

fn verify_keys<S>(gpg_keys: S) -> Result<Vec<String>>
where
    S: AsRef<[String]>,
{
    let gpg_keys = gpg_keys.as_ref();

    let mut keys = Vec::new();
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

    for key in gpg_keys {
        if let Ok(secret_key) = ctx.get_secret_key(key) {
            let user_id = if let Ok(email) = secret_key
                .user_ids()
                .nth(0)
                .with_context(|| "Option did not contain a value.")?
                .email()
            {
                email.to_owned()
            } else {
                key.to_owned()
            };

            keys.push(user_id);
        } else {
            continue;
        }
    }

    if keys.is_empty() || keys.get(0).map(|k| k.is_empty()).unwrap_or(false) {
        Err(PassrsError::NoPrivateKeyFound.into())
    } else {
        Ok(keys)
    }
}

fn git_prep(repo: &Repository) -> Result<(git2::Oid, git2::Signature, Vec<git2::Commit>)> {
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

fn create_store<P, S>(path: P, gpg_keys: S) -> Result<()>
where
    P: AsRef<Path>,
    S: AsRef<[String]>,
{
    let path = path.as_ref();
    let gpg_keys = gpg_keys.as_ref();
    let gpg_ids = verify_keys(gpg_keys)?;

    match fs::metadata(&path) {
        Ok(_) => {}
        Err(_) => fs::create_dir_all(&path)?,
    }

    if let Ok(repo) = Repository::init(&path) {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

        // create .gpg-id
        let gpg_id_path = format!("{}/.gpg-id", path.display());
        let mut file = File::create(&gpg_id_path)?;
        file.write_all(gpg_ids.join("\n").as_bytes())?;

        // create pass .gitattributes
        let gitattributes_path = format!("{}/.gitattributes", path.display());
        let mut file = File::create(&gitattributes_path)?;
        file.write_all(b"*.gpg diff=gpg")?;

        let (tree_id, sig, parents) = git_prep(&repo)?;
        let parents = parents.iter().map(Borrow::borrow).collect::<Vec<_>>();

        // get ready to commit
        let buf = repo.commit_create_buffer(
            &sig,
            &sig,
            &format!("Password store initialized for {}", gpg_keys.join(", ")),
            &repo.find_tree(tree_id)?,
            &parents,
        )?;
        let contents = str::from_utf8(&buf)?.to_string();
        let mut outbuf = Vec::new();

        ctx.set_armor(true);
        ctx.sign(SignMode::Detached, &*buf, &mut outbuf)?;

        let out = str::from_utf8(&outbuf)?;
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
fn create_substore<P, Q, S>(store: P, path: Q, gpg_keys: S) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    S: AsRef<[String]>,
{
    let path = path.as_ref();
    let store = store.as_ref();
    let gpg_keys = gpg_keys.as_ref();
    verify_keys(gpg_keys)?;

    match fs::metadata(&path) {
        Ok(_) => {}
        Err(_) => fs::create_dir_all(&path)?,
    }

    if let Ok(repo) = Repository::open(&store) {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

        // create .gpg-id
        let gpg_id_path = format!("{}/.gpg-id", path.display());
        let mut file = File::create(&gpg_id_path)?;
        file.write_all(gpg_keys.join("\n").as_bytes())?;

        let (tree_id, sig, parents) = git_prep(&repo)?;
        let parents = parents.iter().map(Borrow::borrow).collect::<Vec<_>>();

        // get ready to commit
        let buf = repo.commit_create_buffer(
            &sig,
            &sig,
            &format!(
                "Set GPG ID to {} ({})",
                gpg_keys.join(", "),
                &path.display().to_string()[PASSWORD_STORE_DIR.len()..]
            ),
            &repo.find_tree(tree_id)?,
            &parents,
        )?;
        let contents = str::from_utf8(&buf)?.to_string();
        let mut outbuf = Vec::new();

        ctx.set_armor(true);
        ctx.sign(SignMode::Detached, &*buf, &mut outbuf)?;

        let out = str::from_utf8(&outbuf)?;
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
