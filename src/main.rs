// FIXME: some funcs take Path, some take &str, some take String, and some take
// PathBuf... wtf bro
// FIXME: deal with all unwraps, unless I can guarantee that it won't panic
// FIXME: ensure all/most code is DRY -- Don't Repeat Yourself
// FIXME: document EVERYTHING -- all functions, structs, etc
//   https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments
//   https://doc.rust-lang.org/rustdoc/index.html
//   https://doc.rust-lang.org/stable/reference/comments.html#doc-comments
//   https://doc.rust-lang.org/rust-by-example/meta/doc.html

use failure::Fail;

mod cli;
mod clipboard;
mod consts;
mod error;
mod event;
mod fuzzy;
#[cfg(feature = "otp")]
mod otp;
mod subcmds;
mod tree;
mod ui;
mod util;

// NOTE: external errors
// #[fail(display = "'{}'", _0)]
// GpgError(#[fail(cause)] gpgme::Error),
// #[fail(display = "'{}'", _0)]
// GitError(#[fail(cause)] git2::Error),
// #[fail(display = "'{}'", _0)]
// Io(std::io::Error)
#[derive(Debug, Fail)]
pub(crate) enum PassrsError {
    #[fail(display = "Error: No private key found")]
    NoPrivateKeyFound,
    #[fail(display = "Error: Path '{}' exists", _0)]
    PathExists(String),
    // #[fail(display = "Error: Failed to init git repo")]
    // FailedToInitGitRepo,
    // #[fail(display = "Error: Failed to create directories")]
    // FailedToCreateDirectories,
    #[fail(display = "Error: Failed to open git repo")]
    FailedToOpenGitRepo,
    #[fail(display = "Error: No matches found for search '{}'", _0)]
    NoMatchesFound(String),
    #[fail(display = "Error: No matches found for search '{:?}'", _0)]
    NoMatchesFoundMultiple(Vec<String>),
    #[fail(display = "Error: The entered passwords do not match.")]
    PasswordsDontMatch,
    #[fail(display = "Error: Hashes don't match: '{}' vs '{}'", _0, _1)]
    HashMismatch(String, String),
    #[fail(display = "Error: User aborted")]
    UserAbort,
    #[cfg(feature = "otp")]
    #[fail(display = "Error: URI was not in valid Key Uri Format.\n\
                      See https://github.com/google/google-authenticator/wiki/Key-Uri-Format for more information.")]
    InvalidKeyUri,
    #[cfg(feature = "otp")]
    #[fail(display = "Error: Invalid hash function: '{}'", _0)]
    InvalidHashAlgorithm(String),
    #[fail(display = "Error: No signing key found")]
    NoSigningKeyFound,
    #[fail(display = "Error: Path '{}' does not exist", _0)]
    PathDoesntExist(String),
    #[fail(display = "Error: Sneaky path '{}'", _0)]
    SneakyPath(String),
    #[fail(display = "Error: Store does not exist")]
    StoreDoesntExist,
    #[fail(display = "Error: '{}' is not in the password store", _0)]
    NotInStore(String),
    #[fail(display = "Error: Source is destination")]
    SourceIsDestination,
    #[fail(display = "Error: '{}' is a directory", _0)]
    PathIsDir(String),
    #[fail(display = "Error: Key '{}' is already the signing key", _0)]
    SameKey(String),
    #[fail(display = "Contents unchanged")]
    ContentsUnchanged,
    #[fail(display = "Error: Failed to get contents of clipboard")]
    PasteFailed,
}

fn main() -> failure::Fallible<()> {
    //

    if let Err(err) = cli::opt() {
        // eprintln!("{:?}", e); // this displays the backtrace
        eprintln!("{}", err);
        std::process::exit(1);
    }

    // cli::opt()?;

    Ok(())
}

// TODO: every subcommand should use the following scaffolding before doing
// anything else:
// 1. ensure directories are created
// 2. canonicalize paths
// 3. check for sneaky paths -- what does this entail with Rust? A: just check for ..
//
// 99. commit INSIDE THE SUBCMD
