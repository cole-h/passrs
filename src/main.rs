#![feature(backtrace)]
#![allow(unreachable_code, unused_variables)]
// FIXME: some funcs take Path, some take &str, some take String, and some take
// PathBuf... wtf bro
// FIXME: deal with all unwraps, change to `expect` (or with_context) unless I
// can guarantee that it won't panic
// FIXME: ensure all/most code is DRY -- Don't Repeat Yourself
// FIXME: document EVERYTHING -- all functions, structs, etc
//   https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments
//   https://doc.rust-lang.org/rustdoc/index.html
//   https://doc.rust-lang.org/stable/reference/comments.html#doc-comments
//   https://doc.rust-lang.org/rust-by-example/meta/doc.html

mod cli;
mod clipboard;
mod consts;
mod event;
#[cfg(feature = "otp")]
mod otp;
mod subcmds;
mod tree;
mod ui;
mod util;

// TODO: make ContextExt and make with_context show line numbers
//   https://github.com/dtolnay/anyhow/issues/22
#[derive(Debug, thiserror::Error)]
pub(crate) enum PassrsError {
    #[error("Error: No private key found")]
    NoPrivateKeyFound,
    #[error("Error: No matches found for search '{}'", _0)]
    NoMatchesFound(String),
    #[error("Error: The entered secrets do not match.")]
    SecretsDontMatch,
    #[error("Error: Hashes don't match: '{}' vs '{}'", _0, _1)]
    HashMismatch(String, String),
    #[error("Error: User aborted")]
    UserAbort,
    #[cfg(feature = "otp")]
    #[error("Error: URI was not in valid Key Uri Format.\n\
             See https://github.com/google/google-authenticator/wiki/Key-Uri-Format for more information.")]
    InvalidKeyUri,
    #[cfg(feature = "otp")]
    #[error("Error: Invalid hash function: '{}'", _0)]
    InvalidHashAlgorithm(String),
    #[error("Error: No signing key found")]
    NoSigningKeyFound,
    #[error("Error: Path '{}' does not exist", _0)]
    PathDoesntExist(String),
    #[error("Error: Sneaky path '{}'", _0)]
    SneakyPath(String),
    #[error("Error: Store does not exist")]
    StoreDoesntExist,
    #[error("Error: '{}' is not in the password store", _0)]
    NotInStore(String),
    #[error("Error: Source is destination")]
    SourceIsDestination,
    #[error("Error: '{}' is a directory", _0)]
    PathIsDir(String),
    #[error("Contents unchanged")]
    ContentsUnchanged,
    #[error("Error: Failed to get contents of clipboard")]
    PasteFailed,
}

fn main() {
    if let Err(err) = cli::opt() {
        // eprintln!("{:?}", err.backtrace()); // this displays the backtrace
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

// TODO: every subcommand should use the following scaffolding before doing
// anything else:
// 1. ensure directories are created
// 2. canonicalize paths
// 3. check for sneaky paths -- what does this entail with Rust? A: just check for ..
//
// 99. commit INSIDE THE SUBCMD
