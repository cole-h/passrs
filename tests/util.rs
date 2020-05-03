use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use passrs::consts;
use passrs::util;

fn test_setup() {
    println!("Importing test key");
    Command::new("gpg")
        .args(&["--import", "./tests/passrs@testuser.secret.asc"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn gpg to import secret key");

    println!("Trusting test key");
    Command::new("gpg")
        .arg("--import-ownertrust")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn gpg to import ownertrust")
        .stdin
        .expect("stdin wasn't captured")
        .write_all(b"4B0D9BBAC5C8329C035B125CF6EF0D39C5F84192:6:\n")
        .expect("failed to write to stdin");

    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("PASSWORD_STORE_DIR", "./tests/test_repo");
}

#[test]
fn canonicalize_path() {
    test_setup();

    let paths = [
        "Internet/amazon.com/password",
        &format!("{}/Internet/amazon.com/password", *consts::STORE_STRING),
    ];

    for path in &paths {
        assert_eq!(
            util::canonicalize_path(path).unwrap(),
            PathBuf::from(format!(
                "{}/Internet/amazon.com/password.gpg",
                *consts::STORE_STRING
            ))
        );
    }
}

#[test]
fn exact_path() {
    test_setup();

    let paths = [
        "Internet/amazon.com/password",
        &format!("{}/Internet/amazon.com/password", *consts::STORE_STRING),
    ];

    for path in &paths {
        assert_eq!(
            util::exact_path(path).unwrap(),
            PathBuf::from(format!(
                "{}/Internet/amazon.com/password",
                *consts::STORE_STRING
            ))
        );
    }
}

#[test]
fn check_sneaky_paths() {
    test_setup();

    assert!(util::check_sneaky_paths("../../password").is_err());
    assert!(util::check_sneaky_paths("..").is_err());
    assert!(util::check_sneaky_paths("/../password").is_err());
    assert!(util::check_sneaky_paths("amazon/../password").is_err());
    assert!(util::check_sneaky_paths("password").is_ok());
}

#[test]
fn find_matches() {
    test_setup();

    assert!(util::find_matches(".").unwrap().len() > 0);
    assert!(util::find_matches("a").unwrap().len() == 1);
    assert!(util::find_matches("z").is_err());
}

#[test]
fn decrypt_file_into_bytes() {
    test_setup();

    let file = "./tests/test_repo/a.gpg";
    let contents = util::decrypt_file_into_bytes(&file).unwrap();

    assert_eq!(contents, "eHy;CDpa&4]Sf1g*rx1Zlrig".as_bytes());
}

#[test]
fn decrypt_file_into_strings() {
    test_setup();

    let file = "./tests/test_repo/f.gpg";
    let contents = util::decrypt_file_into_strings(&file).unwrap();
    let mut iter = contents.iter();

    assert!(contents.len() == 6);
    assert_eq!(iter.next(), Some(&String::from("a")));
    assert_eq!(iter.next(), Some(&String::from("b")));
    assert_eq!(iter.next(), Some(&String::from("c")));
    assert_eq!(iter.next(), Some(&String::from("d")));
    assert_eq!(iter.next(), Some(&String::from("e")));
    assert_eq!(iter.next(), Some(&String::from("f")));
    assert_eq!(iter.next(), None);
}

#[test]
fn find_gpg_id() {
    test_setup();

    assert!(util::find_gpg_id("/").is_err());
    assert!(util::find_gpg_id(&*consts::PASSWORD_STORE_DIR).is_ok());
}
