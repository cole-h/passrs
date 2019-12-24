// XXX: I don't know how to create a signed commit with git2-rs

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use failure::Fallible;
use git2::Repository;
#[cfg(feature = "gpg")]
use gpgme::{Context, Protocol};

use crate::consts::DEFAULT_STORE_PATH;
use crate::error::PassrsError;
use crate::utils::*;

// 1. verify provided path
// 2. verify provided key
// 3. setup repo
// TODO: The init command will keep signatures of .gpg-id files up to date.
pub fn init(path: Option<String>, key: String) {
    let path = path.unwrap_or(DEFAULT_STORE_PATH.to_string());

    if verify_path(&path).is_err() {
        eprintln!("Path already exists: {:?}", &path);
        return;
    }
    #[cfg(feature = "gpg")]
    let gpg_id = match verify_key(&key) {
        Ok(gpg_id) => gpg_id,
        Err(e) => {
            eprintln!("Failed to verify key: {:?}", e);
            return;
        }
    };

    #[cfg(not(feature = "gpg"))]
    let gpg_id = String::new();

    let ret = setup_repo(&path, &gpg_id);

    println!("{:?}", ret);
}

// TODO: like gopass, iterate over keys that match
#[cfg(feature = "gpg")]
pub fn verify_key<S>(gpg_id: S) -> Fallible<String>
where
    S: Into<String>,
{
    let key = gpg_id.into();
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

    if let Ok(secret_key) = ctx.get_secret_key(&key) {
        let email = if let Ok(email) = secret_key.user_ids().into_iter().nth(0).unwrap().email() {
            email.to_string()
        } else {
            key
        };

        return Ok(email);
    } else {
        return Err(PassrsError::NoPrivateKeyFound.into());
    }
}

fn git_init(path: &String) -> Fallible<Repository> {
    let repo = match Repository::init(path) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("failed to init git repo: {:?}", e);
            return Err(PassrsError::FailedToInitGitRepo.into());
        }
    };

    Ok(repo)
}

// TODO: error handling
fn setup_repo(path: &String, gpg_id: &String) -> Fallible<()> {
    // FIXME: only used to prevent littering my FS when testing
    match fs::create_dir_all(&path) {
        Ok(_) => {}
        Err(_) => return Err(PassrsError::FailedToCreateDirectories.into()),
    }

    if let Ok(repo) = git_init(&path) {
        // if cfg!(not(debug_assertions))
        // create files
        {
            let gpg_id_path = format!("{}/.gpg-id", path);
            let gitattributes_path = format!("{}/.gitattributes", path);

            // create pass .gpg-id
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
        }

        let mut index = repo.index()?;
        // git add
        index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        // git commit (only if workingdir is dirty)
        // if workingdir is dirty {
        let tree_id = repo.index()?.write_tree()?;
        let sig = repo.signature()?;
        let mut parents = Vec::new();
        if let Some(parent) = repo.head().ok().map(|h| h.target().unwrap()) {
            parents.push(repo.find_commit(parent)?);
        }
        let parents = parents.iter().collect::<Vec<_>>();

        let ret = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "test",
            &repo.find_tree(tree_id)?,
            &parents,
        )?;

        // let buf =
        //     repo.commit_create_buffer(&sig, &sig, "test", &repo.find_tree(tree_id)?, &parents)?;
        // let contents = std::str::from_utf8(&buf).unwrap().to_string();
        // let ret = repo.commit_signed(&contents, r"lol", None)?;

        println!("{:?}", ret);
        // }
    }

    Ok(())
}
