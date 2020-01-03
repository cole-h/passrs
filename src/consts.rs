//! Technically not actual constants, but runtime constants.
use std::env;

use once_cell::sync::Lazy;

pub const DIGITS: &[u8] = b"0123456789"; // [:digit:]
pub const ALPHA_UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"; // [:upper:]
pub const ALPHA_LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz"; // [:lower:]
pub const SPECIAL: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"; // [:punct:]

// pub static VERSION: Lazy<String> = Lazy::new(|| {
//     use structopt::clap::crate_version;

//     let mut ver = crate_version!().to_owned();
//     let commit_hash = env!("GIT_HASH");
//     if commit_hash != "" {
//         ver = format!("{} ({})", ver, commit_hash);
//     }
//     ver
// });

pub static VERSION: Lazy<String> = Lazy::new(|| {
    use structopt::clap::crate_version;

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
        "/usr/bin/vi".to_owned()
    }
});
// FIXME: remove in favor of PASSWORD_STORE_DIR (only used for debugging rn)
pub static DEFAULT_STORE_PATH: Lazy<String> = Lazy::new(|| "/tmp/passrstest/".to_owned());
// env::var("PASSWORD_STORE_DIR").unwrap_or_else(|_| format!("{}/.password-store/", *HOME));
pub static GPG_ID_FILE: Lazy<String> = Lazy::new(|| [&PASSWORD_STORE_DIR, ".gpg-id"].concat());
pub static PASSRS_UNCLIP_HASH: Lazy<String> =
    Lazy::new(|| env::var("PASSRS_UNCLIP_HASH").unwrap_or_default());
pub static PASSRS_GIT_BINARY: Lazy<String> =
    Lazy::new(|| env::var("PASSRS_GIT_BINARY").unwrap_or_else(|_| "/usr/bin/git".to_owned()));

// pass(1)
pub static PASSWORD_STORE_DIR: Lazy<String> =
    Lazy::new(|| env::var("PASSWORD_STORE_DIR").unwrap_or_else(|_| DEFAULT_STORE_PATH.to_owned()));
pub static PASSWORD_STORE_KEY: Lazy<String> =
    Lazy::new(|| env::var("PASSWORD_STORE_KEY").unwrap_or_default());
// TODO: unimplemented
// pub static PASSWORD_STORE_GPG_OPTS: Lazy<String> =
//     Lazy::new(|| env::var("PASSWORD_STORE_GPG_OPTS").unwrap_or_default());
// NOTE: Wayland is the target for this, which doesn't use the X clipboard.
// However, this will be implemented when I get around to cleaning up clipboard.rs
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
// TODO: unimplemented
// pub static PASSWORD_STORE_UMASK: Lazy<String> =
//     Lazy::new(|| env::var("PASSWORD_STORE_UMASK").unwrap_or_else(|_| "022".to_owned()));
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
pub static PASSWORD_STORE_SIGNING_KEY: Lazy<String> = Lazy::new(|| {
    env::var("PASSWORD_STORE_SIGNING_KEY").unwrap_or_else(|_| PASSWORD_STORE_KEY.to_owned())
});
// TODO: decided to say fuck it, remove this
// pub static ref  GREP_OPTIONS: Vec<String> =
//     env::var("GREPOPTIONS").unwrap_or_default().split(' ').map(ToOwned::to_owned).collect::<Vec<_>>();

// lazy_static::lazy_static! {
//     pub static ref VERSION: String = {
//         use structopt::clap::crate_version;
//         let mut ver = crate_version!().to_owned();
//         let commit_hash = env!("GIT_HASH");
//         if commit_hash != "" {
//             ver = format!("{} ({})", ver, commit_hash);
//         }
//         ver
//     };
//     pub static ref HOME: String = env::var("HOME").expect("HOME was not set");
//     pub static ref EDITOR: String = {
//         if let Ok(editor) = env::var("EDITOR") {
//             editor
//         } else if let Ok(visual) = env::var("VISUAL") {
//             visual
//         } else {
//             "/usr/bin/vi".to_owned()
//         }
//     };
//     // FIXME: remove in favor of PASSWORD_STORE_DIR (only used for debugging rn)
//     pub static ref DEFAULT_STORE_PATH: String = "/tmp/passrstest/".to_owned();
//     // env::var("PASSWORD_STORE_DIR").unwrap_or_else(|_| format!("{}/.password-store/", *HOME));
//     pub static ref GPG_ID_FILE: String = [&DEFAULT_STORE_PATH, ".gpg-id"].concat();
//     pub static ref PASSRS_UNCLIP_HASH: String = env::var("PASSRS_UNCLIP_HASH").unwrap_or_default();
//     pub static ref PASSRS_GIT_BINARY: String =
//         env::var("PASSRS_GIT_BINARY").unwrap_or_else(|_| "/usr/bin/git".to_owned());

//     // pass(1)
//     pub static ref PASSWORD_STORE_DIR: String =
//         env::var("PASSWORD_STORE_DIR").unwrap_or_else(|_| DEFAULT_STORE_PATH.to_owned());
//     pub static ref PASSWORD_STORE_KEY: String = env::var("PASSWORD_STORE_KEY").unwrap_or_default();
//     pub static ref PASSWORD_STORE_GPG_OPTS: String =
//         env::var("PASSWORD_STORE_GPG_OPTS").unwrap_or_default();
//     // NOTE: Wayland is the target for this, which doesn't use the X clipboard.
//     // However, this will be implemented when I get around to cleaning up clipboard.rs
//     pub static ref PASSWORD_STORE_X_SELECTION: String = match env::var("PASSWORD_STORE_X_SELECTION") {
//         Ok(sel) => match sel.as_ref() {
//             "p" | "primary" => sel.to_owned(),
//             "sec" | "secondary" => sel.to_owned(),
//             _ => "clipboard".to_owned(),
//         },
//         Err(_) => "clipboard".to_owned()
//     };
//     pub static ref PASSWORD_STORE_CLIP_TIME: String =
//         env::var("PASSWORD_STORE_CLIP_TIME").unwrap_or_else(|_| "45".to_owned());
//     pub static ref PASSWORD_STORE_UMASK: String =
//         env::var("PASSWORD_STORE_UMASK").unwrap_or_else(|_| "022".to_owned());
//     pub static ref PASSWORD_STORE_GENERATED_LENGTH: usize = env::var("PASSWORD_STORE_GENERATED_LENGTH")
//         .unwrap_or_else(|_| "24".to_owned())
//         .parse::<usize>()
//         .unwrap();
//     pub static ref PASSWORD_STORE_CHARACTER_SET: Vec<u8> = {
//         match env::var("PASSWORD_STORE_CHARACTER_SET") {
//             Ok(set) => set.bytes().collect::<Vec<_>>(),
//             Err(_) => [DIGITS, ALPHA_LOWER, ALPHA_UPPER, SPECIAL].concat(),
//         }
//     };
//     pub static ref PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS: Vec<u8> = {
//         match env::var("PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS") {
//             Ok(set) => set.bytes().collect::<Vec<_>>(),
//             Err(_) => [DIGITS, ALPHA_LOWER, ALPHA_UPPER].concat(),
//         }
//     };
//     pub static ref PASSWORD_STORE_SIGNING_KEY: String =
//         env::var("PASSWORD_STORE_SIGNING_KEY").unwrap_or_else(|_| PASSWORD_STORE_KEY.to_owned());
//     // TODO: decided to say fuck it, remove this
//     // pub static ref  GREP_OPTIONS: Vec<String> =
//     //     env::var("GREPOPTIONS").unwrap_or_default().split(' ').map(ToOwned::to_owned).collect::<Vec<_>>();
// }
