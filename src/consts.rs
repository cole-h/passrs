//! Technically not actual constants, but runtime constants.

use lazy_static::lazy_static;

pub const DIGITS: &[u8] = b"0123456789";
pub const ALPHA_UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const ALPHA_LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
pub const SPECIAL: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

lazy_static! {
    pub static ref VERSION: String = {
        use structopt::clap::crate_version;
        let mut ver = crate_version!().to_owned();
        let commit_hash = env!("GIT_HASH");
        if commit_hash != "" {
            ver = format!("{} ({})", ver, commit_hash);
        }
        ver
    };
    pub static ref HOME: String = envmnt::get_or_panic("HOME");
    pub static ref EDITOR: String = envmnt::get_any(&vec!["EDITOR", "VISUAL"], "/usr/bin/vim");
    // FIXME: remove in favor of PASSWORD_STORE_DIR (only used for debugging rn)
    pub static ref DEFAULT_STORE_PATH: String =
        "/tmp/passrstest/".to_owned();
        // envmnt::get_or("PASSWORD_STORE_DIR", &format!("{}/.password-store/", *HOME));
    pub static ref GPG_ID_FILE: String = [&DEFAULT_STORE_PATH, ".gpg-id"].concat();
    pub static ref PASSRS_UNCLIP_HASH: String = envmnt::get_or("PASSRS_UNCLIP_HASH", "");
    pub static ref PASSRS_GIT_BINARY: String = envmnt::get_or("PASSRS_GIT_BINARY", "/usr/bin/git");

    // pass(1)
    pub static ref PASSWORD_STORE_DIR: String = envmnt::get_or("PASSWORD_STORE_DIR", &DEFAULT_STORE_PATH);
    pub static ref PASSWORD_STORE_KEY: String = envmnt::get_or("PASSWORD_STORE_KEY", "");
    pub static ref PASSWORD_STORE_GPG_OPTS: String = envmnt::get_or("PASSWORD_STORE_GPG_OPTS", "");
    // NOTE: Wayland is the target for this, which doesn't use the X clipboard.
    // However, this will be implemented when I get around to cleaning up clipboard.rs
    // pub static ref PASSWORD_STORE_X_SELECTION: String =
    //     envmnt::get_or("PASSWORD_STORE_X_SELECTION", "");
    pub static ref PASSWORD_STORE_CLIP_TIME: String =
        envmnt::get_or("PASSWORD_STORE_CLIP_TIME", "45");
    pub static ref PASSWORD_STORE_UMASK: String = envmnt::get_or("PASSWORD_STORE_UMASK", "077");
    pub static ref PASSWORD_STORE_GENERATED_LENGTH: usize =
        envmnt::get_or("PASSWORD_STORE_GENERATED_LENGTH", "24").parse::<usize>().unwrap();
    pub static ref PASSWORD_STORE_CHARACTER_SET: Vec<u8> = {
        match std::env::var("PASSWORD_STORE_CHARACTER_SET") {
            Ok(set) => set.bytes().collect::<Vec<_>>(),
            Err(_) => [DIGITS, ALPHA_LOWER, ALPHA_UPPER, SPECIAL].concat(),
        }
    };
    pub static ref PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS: Vec<u8> = {
        match std::env::var("PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS") {
            Ok(set) => set.bytes().collect::<Vec<_>>(),
            Err(_) => [DIGITS, ALPHA_LOWER, ALPHA_UPPER].concat(),
        }
    };
    pub static ref PASSWORD_STORE_SIGNING_KEY: String =
        envmnt::get_or("PASSWORD_STORE_SIGNING_KEY", &PASSWORD_STORE_KEY);
    pub static ref GREP_OPTIONS: Vec<String> = envmnt::get_list("GREP_OPTIONS").unwrap_or_default();
}
