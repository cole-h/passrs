use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use failure::{err_msg, Fallible};
use git2::Repository;
use gpgme::{Context, Protocol, SignMode};

use crate::consts::{PASSWORD_STORE_DIR, PASSWORD_STORE_KEY};
use crate::error::PassrsError;
use crate::util;

// 1. verify provided path
// 2. verify provided key
// 3. setup repo
// TODO: The init command will keep signatures of .gpg-id files up to date.
pub fn init(path: Option<String>, key: Option<String>) -> Fallible<()> {
    let path = path.unwrap_or_else(|| PASSWORD_STORE_DIR.to_owned());
    let key = key.unwrap_or_else(|| PASSWORD_STORE_KEY.to_owned());

    util::path_exists(&path)?; // TODO: init actually creates substores, or
                               // re-encrypts everything using the specified key
    setup_repo(path, key)?;
    // update .gpg-id file

    Ok(())
}

fn git_init(path: &str) -> Fallible<Repository> {
    let repo = match Repository::init(path) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("failed to init git repo: {:?}", e);
            return Err(PassrsError::FailedToInitGitRepo.into());
        }
    };

    Ok(repo)
}

// TODO: like gopass, iterate over keys that match
pub fn verify_key<S>(gpg_key: S) -> Fallible<String>
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

fn setup_repo(path: String, gpg_key: String) -> Fallible<()> {
    let gpg_id = verify_key(&gpg_key)?;

    match fs::create_dir_all(&path) {
        Ok(_) => {}
        Err(_) => return Err(PassrsError::FailedToCreateDirectories.into()),
    }

    if let Ok(repo) = git_init(&path) {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

        let gpg_id_path = format!("{}/.gpg-id", path);
        let gitattributes_path = format!("{}/.gitattributes", path);

        // create .gpg-id
        let path = Path::new(&gpg_id_path);
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(e) => panic!("failed to create file {:?}: {:?}", path, e),
        };
        match file.write_all(gpg_id.as_bytes()) {
            Ok(_) => {}
            Err(e) => panic!("failed to write to file {:?}: {:?}", file, e),
        }

        // create pass .gitattributes
        let path = Path::new(&gitattributes_path);
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(e) => panic!("failed to create file {:?}: {:?}", path, e),
        };
        match file.write_all(b"*.gpg diff=gpg") {
            Ok(_) => {}
            Err(e) => panic!("failed to write to file {:?}: {:?}", file, e),
        }

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
        ctx.sign(
            SignMode::Detached,
            buf.as_str()
                .ok_or_else(|| err_msg("Buffer was not valid UTF-8"))?,
            &mut outbuf,
        )?;

        let out = std::str::from_utf8(&outbuf)?;
        let ret = repo.commit_signed(&contents, &out, Some("gpgsig"))?;

        // TODO: verify there are no side-effects to this
        // If you use "HEAD" as the reference to change, master isn't updated.
        // Short refs don't work.
        match repo.reference("refs/heads/master", ret, false, "TODO: init message") {
            Ok(reference) => reference,
            Err(_) => repo.reference("refs/heads/master", ret, true, "TODO: reinit message")?,
        };

        dbg!(ret);
    }

    Ok(())
}
