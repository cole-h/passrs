//! Runtime constants
//!
//! # consts
//!
//! This module houses constants used throughout the code. Many of these are
//! just lazily-evaluated environment variables.

use std::env;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use structopt::clap::crate_version;

pub const DIGITS: &[u8] = b"0123456789"; // [:digit:]
pub const ALPHA_UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"; // [:upper:]
pub const ALPHA_LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz"; // [:lower:]
pub const SPECIAL: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"; // [:punct:]

pub static VERSION: Lazy<String> = Lazy::new(|| {
    let ver = crate_version!().to_owned();
    let commit_hash = env!("GIT_HASH");

    if !commit_hash.is_empty() {
        format!("{} ({})", ver, commit_hash)
    } else {
        ver
    }
});
pub static EDITOR: Lazy<String> = Lazy::new(|| {
    let editor = if let Ok(editor) = env::var("EDITOR") {
        editor
    } else if let Ok(visual) = env::var("VISUAL") {
        visual
    } else {
        String::from("/usr/bin/vi")
    };

    if editor.is_empty() {
        String::from("/usr/bin/vi")
    } else {
        editor
    }
});
pub static HOME: Lazy<String> = Lazy::new(|| env::var("HOME").expect("HOME was not set"));
pub static GPG_ID_FILE: Lazy<PathBuf> = Lazy::new(|| PASSWORD_STORE_DIR.join(".gpg-id"));
pub static PASSRS_UNCLIP_HASH: Lazy<String> =
    Lazy::new(|| env::var("PASSRS_UNCLIP_HASH").unwrap_or_default());
pub static PASSRS_GIT_BINARY: Lazy<String> =
    Lazy::new(|| env::var("PASSRS_GIT_BINARY").unwrap_or_else(|_| String::from("/usr/bin/git")));
pub static STORE_STRING: Lazy<String> = Lazy::new(|| PASSWORD_STORE_DIR.display().to_string());
// if the store_string doesn't end with a '/', account for that (subpaths *will* have the '/')
pub static STORE_LEN: Lazy<usize> = Lazy::new(|| {
    if STORE_STRING.ends_with('/') {
        STORE_STRING.len()
    } else {
        STORE_STRING.len() + 1
    }
});

// pass(1)
pub static PASSWORD_STORE_DIR: Lazy<PathBuf> = Lazy::new(|| match env::var("PASSWORD_STORE_DIR") {
    Ok(store) => PathBuf::from(store),
    Err(_) => PathBuf::from(format!("{}/.password-store/", *HOME)),
});
pub static PASSWORD_STORE_KEY: Lazy<Vec<String>> = Lazy::new(|| {
    env::var("PASSWORD_STORE_KEY")
        .unwrap_or_default()
        .split(' ')
        .filter(|&e| !e.is_empty())
        .map(ToOwned::to_owned)
        .collect()
});
pub static PASSWORD_STORE_X_SELECTION: Lazy<String> =
    Lazy::new(|| match env::var("PASSWORD_STORE_X_SELECTION") {
        Ok(sel) => match sel.as_ref() {
            "p" | "primary" => sel.to_owned(),
            "sec" | "secondary" => sel.to_owned(),
            _ => "clipboard".to_owned(),
        },
        Err(_) => "clipboard".to_owned(),
    });
pub static PASSWORD_STORE_CLIP_TIME: Lazy<String> =
    Lazy::new(|| env::var("PASSWORD_STORE_CLIP_TIME").unwrap_or_else(|_| "45".to_owned()));
pub static PASSWORD_STORE_UMASK: Lazy<u32> = Lazy::new(|| {
    u32::from_str_radix(
        &env::var("PASSWORD_STORE_UMASK").unwrap_or_else(|_| "077".to_owned()),
        8,
    )
    .expect("umask was not valid octal")
});
pub static PASSWORD_STORE_GENERATED_LENGTH: Lazy<usize> = Lazy::new(|| {
    env::var("PASSWORD_STORE_GENERATED_LENGTH")
        .unwrap_or_else(|_| "24".to_owned())
        .parse::<usize>()
        .expect("length was not a usize")
});
pub static PASSWORD_STORE_CHARACTER_SET: Lazy<Vec<u8>> =
    Lazy::new(|| match env::var("PASSWORD_STORE_CHARACTER_SET") {
        Ok(set) => set.bytes().collect(),
        Err(_) => [DIGITS, ALPHA_LOWER, ALPHA_UPPER, SPECIAL].concat(),
    });
pub static PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS: Lazy<Vec<u8>> =
    Lazy::new(
        || match env::var("PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS") {
            Ok(set) => set.bytes().collect(),
            Err(_) => [DIGITS, ALPHA_LOWER, ALPHA_UPPER].concat(),
        },
    );
pub static PASSWORD_STORE_SIGNING_KEY: Lazy<Vec<String>> = Lazy::new(|| {
    env::var("PASSWORD_STORE_SIGNING_KEY")
        .unwrap_or_default()
        .split(' ')
        .filter(|&e| !e.is_empty())
        .map(ToOwned::to_owned)
        .collect()
});
