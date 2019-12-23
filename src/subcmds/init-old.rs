use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use gpgme::{Context, Protocol};

use crate::error::PassrsError;
use crate::subcmds::run_command;
use crate::Result;

// 1. verify provided path
// 2. verify provided key
// 3. setup repo
pub fn init(path: String, key: String) {
    if verify_path(&path).is_err() {
        eprintln!("Path already exists: {:?}", &path);
        return;
    }
    let gpg_id = match verify_key(&key) {
        Ok(gpg_id) => gpg_id,
        Err(e) => {
            eprintln!("Failed to verify key: {:?}", e);
            return;
        }
    };

    let ret = setup_repo(&path, &gpg_id);

    println!("{:?}", ret);
}

/// Returns `()` if path is valid, or an error if path is invalid.
fn verify_path(path: &String) -> Result<()> {
    let meta = fs::metadata(path);
    if meta.is_ok() {
        return Err(Box::new(PassrsError::PathExists));
    }
    //   check_sneaky_paths(&path)
    //   check if path already exists
    Ok(())
}

// TODO: like gopass, iterate over keys that match
pub fn verify_key<S>(gpg_id: S) -> Result<String>
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
        return Err(Box::new(PassrsError::NoPrivateKeyFound));
    }
}

fn git_init(path: &String) {
    run_command("/usr/bin/git", vec!["-C", path, "init"]);
}

// TODO: error handling
fn setup_repo(path: &String, gpg_id: &String) -> Result<()> {
    // FIXME: only used to prevent littering my FS when testing
    match fs::create_dir_all(&path) {
        Ok(_) => {}
        Err(_) => return Err(Box::new(PassrsError::FailedToCreateDirectories)),
    }

    git_init(&path);
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

    // git add
    run_command("/usr/bin/git", vec!["add", "-A"]);
    // git commit (only if workingdir is dirty)
    // if workingdir is dirty {
    let message = "";
    if !run_command(
        "/usr/bin/git",
        vec!["diff-index", "--quiet", "HEAD", "--", "2>/dev/null"],
    )
    .success()
    {
        run_command("/usr/bin/git", vec!["commit", "-m", message, "-S"]);
    }
    // }

    Ok(())
}
