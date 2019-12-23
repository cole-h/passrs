use envmnt;
use lazy_static::lazy_static;
use structopt::clap::crate_version;

pub const DIGITS: &'static [u8] = b"0123456789";
pub const ALPHA_UPPER: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const ALPHA_LOWER: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz";
pub const SPECIAL: &'static [u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

lazy_static! {
    pub static ref VERSION: String = {
        let mut ver = crate_version!().to_owned();
        let commit_hash = env!("GIT_HASH");
        if commit_hash != "" {
            ver = format!("{} ({})", ver, commit_hash);
        }

        ver
    };
    pub static ref EDITOR: String = envmnt::get_any(&vec!["EDITOR", "VISUAL"], "/usr/bin/vim");
    pub static ref DEFAULT_STORE_PATH: String = {
        let home = envmnt::get_or_panic("HOME");
        envmnt::get_or("PASSWORD_STORE_DIR", &format!("{}/.password-store/", home))
    };
    pub static ref PASSRS_UNCLIP_HASH: String = envmnt::get_or("PASSRS_UNCLIP_HASH", "");

    // pass(1)
    pub static ref PASSWORD_STORE_DIR: String = envmnt::get_or("PASSWORD_STORE_DIR", &DEFAULT_STORE_PATH);
    pub static ref PASSWORD_STORE_KEY: String = envmnt::get_or("PASSWORD_STORE_KEY", "");
    pub static ref PASSWORD_STORE_GPG_OPTS: String = envmnt::get_or("PASSWORD_STORE_GPG_OPTS", "");
    // TODO: maybe make noop idk
    // pub static ref PASSWORD_STORE_X_SELECTION: String =
    //     envmnt::get_or("PASSWORD_STORE_X_SELECTION", "");
    pub static ref PASSWORD_STORE_CLIP_TIME: String =
        envmnt::get_or("PASSWORD_STORE_CLIP_TIME", "");
    pub static ref PASSWORD_STORE_UMASK: String = envmnt::get_or("PASSWORD_STORE_UMASK", "");
    // TODO: deprecate
    // pub static ref PASSWORD_STORE_GENERATED_LENGTH: String =
    //     envmnt::get_or("PASSWORD_STORE_GENERATED_LENGTH", "");
    pub static ref PASSWORD_STORE_CHARACTER_SET: String =
        envmnt::get_or("PASSWORD_STORE_CHARACTER_SET", "");
    pub static ref PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS: String =
        envmnt::get_or("PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS", "");
    // TODO: probably not gona happen
    // pub static ref PASSWORD_STORE_ENABLE_EXTENSIONS: String =
    //     envmnt::get_or("PASSWORD_STORE_ENABLE_EXTENSIONS", "");
    // TODO: probably not gona happen
    // pub static ref PASSWORD_STORE_EXTENSIONS_DIR: String =
    //     envmnt::get_or("PASSWORD_STORE_EXTENSIONS_DIR", "");
    pub static ref PASSWORD_STORE_SIGNING_KEY: String =
        envmnt::get_or("PASSWORD_STORE_SIGNING_KEY", "");
}
