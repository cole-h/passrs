use std::path::PathBuf;

use passrs::consts;
use passrs::util;

#[test]
fn canonicalize_path() {
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
    assert!(util::check_sneaky_paths("../../password").is_err());
    assert!(util::check_sneaky_paths("..").is_err());
    assert!(util::check_sneaky_paths("/../password").is_err());
    assert!(util::check_sneaky_paths("amazon/../password").is_err());
}

#[test]
fn find_target_single() {
    assert!(util::find_target_single(".").unwrap().len() > 0);
}
