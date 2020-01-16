//! Technically not actual constants, but runtime constants.
use std::env;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use structopt::clap::crate_version;

pub const DIGITS: &[u8] = b"0123456789"; // [:digit:]
pub const ALPHA_UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"; // [:upper:]
pub const ALPHA_LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz"; // [:lower:]
pub const SPECIAL: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"; // [:punct:]

pub static VERSION: Lazy<String> = Lazy::new(|| {
    let mut ver = crate_version!().to_owned();
    let commit_hash = env!("GIT_HASH");

    if commit_hash != "" {
        ver = format!("{} ({})", ver, commit_hash);
    }

    ver
});

pub static HOME: Lazy<String> = Lazy::new(|| env::var("HOME").expect("HOME was not set"));
pub static EDITOR: Lazy<String> = Lazy::new(|| {
    if let Ok(editor) = env::var("EDITOR") {
        editor
    } else if let Ok(visual) = env::var("VISUAL") {
        visual
    } else {
        String::from("/usr/bin/vi")
    }
});

pub static GPG_ID_FILE: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from(&*PASSWORD_STORE_DIR.join(".gpg-id")));
pub static PASSRS_UNCLIP_HASH: Lazy<String> =
    Lazy::new(|| env::var("PASSRS_UNCLIP_HASH").unwrap_or_default());
pub static PASSRS_GIT_BINARY: Lazy<String> =
    Lazy::new(|| env::var("PASSRS_GIT_BINARY").unwrap_or_else(|_| String::from("/usr/bin/git")));
// used to allow comparisons with the password store dir without having to convert it w/ to_string()
pub static STORE_STRING: Lazy<String> = Lazy::new(|| PASSWORD_STORE_DIR.display().to_string());
// used to prevent the need to litter `PASSWORD_STORE_DIR.display().to_string().len()` everywhere
pub static STORE_LEN: Lazy<usize> = Lazy::new(|| STORE_STRING.len());

// pass(1)
pub static PASSWORD_STORE_DIR: Lazy<PathBuf> = Lazy::new(|| match env::var("PASSWORD_STORE_DIR") {
    Ok(store) => PathBuf::from(store),
    Err(_) => PathBuf::from(format!("{}/.password-store/", *HOME)),
});
pub static PASSWORD_STORE_KEY: Lazy<Vec<String>> = Lazy::new(|| {
    let keys = env::var("PASSWORD_STORE_KEY").unwrap_or_default();
    keys.split(' ')
        .filter(|&e| e != "")
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>()
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
    let umask = env::var("PASSWORD_STORE_UMASK").unwrap_or_else(|_| "077".to_owned());
    u32::from_str_radix(&umask, 8).unwrap()
});
pub static PASSWORD_STORE_GENERATED_LENGTH: Lazy<usize> = Lazy::new(|| {
    env::var("PASSWORD_STORE_GENERATED_LENGTH")
        .unwrap_or_else(|_| "24".to_owned())
        .parse::<usize>()
        .unwrap()
});
pub static PASSWORD_STORE_CHARACTER_SET: Lazy<Vec<u8>> =
    Lazy::new(|| match env::var("PASSWORD_STORE_CHARACTER_SET") {
        Ok(set) => set.bytes().collect::<Vec<_>>(),
        Err(_) => [DIGITS, ALPHA_LOWER, ALPHA_UPPER, SPECIAL].concat(),
    });
pub static PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS: Lazy<Vec<u8>> =
    Lazy::new(
        || match env::var("PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS") {
            Ok(set) => set.bytes().collect::<Vec<_>>(),
            Err(_) => [DIGITS, ALPHA_LOWER, ALPHA_UPPER].concat(),
        },
    );
pub static PASSWORD_STORE_SIGNING_KEY: Lazy<Vec<String>> = Lazy::new(|| {
    let keys = env::var("PASSWORD_STORE_SIGNING_KEY").unwrap_or_default();
    keys.split(' ')
        .filter(|&e| e != "")
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>()
});
