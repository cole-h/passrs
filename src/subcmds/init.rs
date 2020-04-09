use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::str;

use anyhow::{Context as _, Result};
use git2::{Commit, Oid, Repository, Signature};
use gpgme::{Context, Protocol};

use crate::consts::{
    PASSWORD_STORE_DIR, PASSWORD_STORE_KEY, PASSWORD_STORE_SIGNING_KEY, PASSWORD_STORE_UMASK,
    STORE_LEN, STORE_STRING,
};
use crate::util;
use crate::PassrsError;

pub(crate) fn init(keys: Vec<String>, path: Option<String>) -> Result<()> {
    let store = &*PASSWORD_STORE_DIR;
    let keys = if keys.is_empty() {
        &*PASSWORD_STORE_KEY
    } else {
        &keys
    };

    if !util::path_exists(&store)? || util::verify_store_exists().is_err() {
        if let Some(path) = path {
            writeln!(
                io::stderr(),
                "Ignoring path {}; creating store at {}",
                path,
                *STORE_STRING
            )?;
        }
        if keys.is_empty() {
            return Err(PassrsError::StoreDoesntExist.into());
        }

        self::create_store(&store, &keys)?;

        let list = &keys.join(", ");
        let keys = if keys.len() > 1 { &list } else { &keys[0] };

        writeln!(io::stdout(), "Password store initialized for {}", keys)?;
    } else if keys.is_empty() {
        // Although pass allows the deinitialization of a store, we don't
        //   https://git.zx2c4.com/password-store/commit/src/password-store.sh?id=0f0483f789e4819b029cf2f9d8168a6172da4d92
        return Err(PassrsError::NoPrivateKeyFound.into());
    } else if let Some(path) = path {
        let substore_path = util::exact_path(&path)?;

        if !util::path_exists(&substore_path)? {
            self::create_substore(&store, &substore_path, &keys)?;

            let list = &keys.join(", ");
            let keys = if keys.len() > 1 { &list } else { &keys[0] };

            writeln!(
                io::stdout(),
                "Password store initialized for {} ({})",
                &keys,
                &path
            )?;
        } else {
            util::recrypt_dir(&substore_path, Some(keys))?;

            let list = &keys.join(", ");
            let new_keys = if keys.len() > 1 { &list } else { &keys[0] };

            self::update_key(&substore_path, &keys)?;
            util::commit(
                None::<&[PathBuf]>,
                format!(
                    "Re-encrypt {} using new GPG ID {}",
                    &substore_path.display().to_string()[*STORE_LEN..],
                    &new_keys,
                ),
            )
            .with_context(|| "Failed to commit re-encrypting substore")?;
        }
    } else {
        util::recrypt_dir(&store, Some(keys))?;

        let list = &keys.join(", ");
        let new_keys = if keys.len() > 1 { &list } else { &keys[0] };

        self::update_key(&store, &keys)?;
        util::commit(
            None::<&[PathBuf]>,
            format!("Re-encrypt password store using new GPG ID {}", &new_keys),
        )
        .with_context(|| "Failed to commit re-encrypted store")?;
    }

    Ok(())
}

fn update_key<K, P>(path: P, keys: K) -> Result<()>
where
    P: AsRef<Path>,
    K: AsRef<[String]>,
{
    let path = path.as_ref();
    let keys = keys.as_ref();

    if keys.is_empty() {
        return Err(PassrsError::NoPrivateKeyFound.into());
    }

    let gpg_ids = verify_keys(keys)?;
    let gpg_id_file = path.join(".gpg-id");
    let gpg_id_sigfile = format!("{}.sig", gpg_id_file.display());
    let mut file = fs::OpenOptions::new()
        .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
        .truncate(true)
        .write(true)
        .create(true)
        .open(&gpg_id_file)?;

    file.write_all(gpg_ids.join("\n").as_bytes())?;

    if PASSWORD_STORE_SIGNING_KEY.is_empty() {
        if PathBuf::from(&gpg_id_sigfile).exists() {
            fs::remove_file(&gpg_id_sigfile)?;
        }
    } else {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
        let mut outbuf: Vec<u8> = Vec::new();
        let mut file = fs::OpenOptions::new()
            .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
            .truncate(true)
            .write(true)
            .create(true)
            .open(&gpg_id_sigfile)?;
        let signing_keys: Vec<gpgme::Key> = PASSWORD_STORE_SIGNING_KEY
            .iter()
            .map(|k| ctx.get_key(k))
            .filter_map(|k| k.ok())
            .collect();

        for key in signing_keys {
            ctx.add_signer(&key)
                .with_context(|| format!("Failed to add key {:?} as signer", key.id()))?;
        }

        ctx.sign_detached(gpg_ids.join("\n").as_bytes(), &mut outbuf)?;
        file.write_all(&outbuf)?;
    }

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
            let email = secret_key
                .user_ids()
                .next()
                .with_context(|| "Option did not contain a value.")?
                .email();
            let user_id = match email {
                Ok(email) => email.to_owned(),
                _ => key.to_owned(),
            };

            keys.push(user_id);
        } else {
            continue;
        }
    }

    if keys.is_empty() {
        Err(PassrsError::NoPrivateKeyFound.into())
    } else {
        Ok(keys)
    }
}

fn git_prep<P, V>(repo: &Repository, files: V) -> Result<(Oid, Signature, Vec<Commit>)>
where
    V: AsRef<[P]>,
    P: AsRef<Path>,
{
    let mut index = repo.index()?;
    let mut pathspecs = Vec::new();

    for path in files.as_ref() {
        let path = path.as_ref();
        let path = if path.starts_with(&*STORE_STRING) {
            PathBuf::from(&path.display().to_string()[*STORE_LEN..])
        } else {
            path.to_path_buf()
        };

        pathspecs.push(path);
    }

    index.add_all(pathspecs, git2::IndexAddOption::CHECK_PATHSPEC, None)?;
    index.write()?;

    let tree_id = repo.index()?.write_tree()?;
    let sig = repo.signature()?;
    let mut parents = Vec::new();

    if let Some(parent) = repo
        .head()
        .ok()
        .map(|h| h.target().expect("HEAD had no target"))
    {
        parents.push(repo.find_commit(parent)?);
    }

    Ok((tree_id, sig, parents))
}

fn create_store<P, S>(path: P, gpg_keys: S) -> Result<()>
where
    P: AsRef<Path>,
    S: AsRef<[String]>,
{
    let path = path.as_ref();
    let gpg_keys = gpg_keys.as_ref();
    let gpg_ids = self::verify_keys(gpg_keys)?;

    if fs::metadata(&path).is_err() {
        fs::create_dir_all(&path)?;
        util::set_permissions_recursive(&path)?;
    }

    // NOTE: pass doesn't actually commit until the user runs `pass git init`,
    // which is intercepted... I don't like this and thus force the creation of
    // a git repository
    if let Ok(repo) = Repository::init(&path) {
        let gpg_id_path = path.join(".gpg-id");
        let mut file = fs::OpenOptions::new()
            .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
            .write(true)
            .create(true)
            .open(&gpg_id_path)?;

        file.write_all(gpg_ids.join("\n").as_bytes())?;

        let gitattributes_path = path.join(".gitattributes");
        let mut file = fs::OpenOptions::new()
            .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
            .write(true)
            .create(true)
            .open(&gitattributes_path)?;

        file.write_all(b"*.gpg diff=gpg")?;

        let mut config = repo.config()?;

        // NOTE: These are the default GPG_OPTS as defined in `pass`. Since we
        // don't call out to gpg, we can't easily support PASSWORD_STORE_GPG_OPTS.
        config.set_str(
            "diff.gpg.textconv",
            "gpg2 --decrypt --quiet --yes --compress-algo=none --no-encrypt-to --batch --use-agent",
        )?;
        config.set_bool("diff.gpg.binary", true)?;

        let (tree_id, sig, parents) = git_prep(&repo, &[gpg_id_path, gitattributes_path])?;
        let parents: Vec<&Commit> = parents.iter().collect();
        let list = &gpg_keys.join(", ");
        let keys = if gpg_keys.len() > 1 {
            &list
        } else {
            &gpg_keys[0]
        };
        let commit_message = format!("Password store initialized for {}", keys);
        let ref_message = format!("commit (initial): {}", &commit_message);

        let commit = if config.get_bool("commit.gpgsign")? {
            let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
            let buf = repo.commit_create_buffer(
                &sig,
                &sig,
                &commit_message,
                &repo.find_tree(tree_id)?,
                &parents,
            )?;
            let contents = str::from_utf8(&buf)?.to_string();
            let mut outbuf = Vec::new();

            ctx.set_armor(true);
            ctx.sign_detached(&*buf, &mut outbuf)?;

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

        repo.reference("refs/heads/master", commit, true, &ref_message)?;
    };

    Ok(())
}

fn create_substore<P, Q, S>(store: P, path: Q, gpg_keys: S) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    S: AsRef<[String]>,
{
    let path = path.as_ref();
    let store = store.as_ref();
    let gpg_keys = gpg_keys.as_ref();

    self::verify_keys(&gpg_keys)?;

    if fs::metadata(&path).is_err() {
        fs::create_dir_all(&path)?;
        util::set_permissions_recursive(&path)?;
    }

    if let Ok(repo) = Repository::open(&store) {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

        let gpg_id_path = path.join(".gpg-id");
        let mut file = fs::OpenOptions::new()
            .mode(0o666 - (0o666 & *PASSWORD_STORE_UMASK))
            .write(true)
            .create(true)
            .open(&gpg_id_path)?;

        file.write_all(gpg_keys.join("\n").as_bytes())?;

        let (tree_id, sig, parents) = git_prep(&repo, [&gpg_id_path])?;
        let parents: Vec<&Commit> = parents.iter().collect();
        let config = repo.config()?;
        let commit_message = format!(
            "Set GPG ID to {} ({})",
            gpg_keys.join(", "),
            &path.display().to_string()[*STORE_LEN..]
        );
        let ref_message = format!("commit: {}", &commit_message);
        let commit = if config.get_bool("commit.gpgsign")? {
            let buf = repo.commit_create_buffer(
                &sig,
                &sig,
                &commit_message,
                &repo.find_tree(tree_id)?,
                &parents,
            )?;
            let contents = str::from_utf8(&buf)?.to_string();
            let mut outbuf = Vec::new();

            ctx.set_armor(true);
            ctx.sign_detached(&*buf, &mut outbuf)?;

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

        repo.reference("refs/heads/master", commit, true, &ref_message)?;
    }

    Ok(())
}
