#![allow(unreachable_code, unused_variables)]
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
    #[error("Error: No matches found for search '{0}'")]
    NoMatchesFound(String),
    #[error("Error: The entered secrets do not match.")]
    SecretsDontMatch,
    #[error("Error: Hashes don't match: '{0}' vs '{1}'")]
    HashMismatch(String, String),
    #[error("Error: User aborted")]
    UserAbort,
    #[cfg(feature = "otp")]
    #[error("Error: URI was not in valid Key Uri Format.\n\
             See https://github.com/google/google-authenticator/wiki/Key-Uri-Format for more information.")]
    InvalidKeyUri,
    #[cfg(feature = "otp")]
    #[error("Error: Invalid hash function: '{0}'")]
    InvalidHashAlgorithm(String),
    #[error("Error: No signing key found")]
    NoSigningKeyFound,
    #[error("Error: Path '{0}' does not exist")]
    PathDoesntExist(String),
    #[error("Error: Sneaky path '{0}'")]
    SneakyPath(String),
    #[error("Error: Store does not exist")]
    StoreDoesntExist,
    #[error("Error: '{0}' is not in the password store")]
    NotInStore(String),
    #[error("Error: Source is destination")]
    SourceIsDestination,
    #[error("Error: '{0}' is a directory")]
    PathIsDir(String),
    #[error("Contents unchanged")]
    ContentsUnchanged,
    #[error("Error: Failed to get contents of clipboard")]
    PasteFailed,
    #[error("Error: No `.gpg-id` was found in '{0}'")]
    NoGpgIdFile(String),
    #[error("Error: Signature for '{0}' does not exist")]
    MissingSignature(String),
    #[error("Error: Signature for '{0}' does not match")]
    BadSignature(String),
    #[error("Error: Failed to set contents of clipboard")]
    ClipFailed,
}

fn main() {
    if let Err(err) = cli::opt() {
        // eprintln!("{:?}", err.backtrace()); // this displays the backtrace
        //                                     // (which requires a nightly compiler)

        eprint!("{:?}", err);
        err.chain().skip(1).for_each(|e| eprint!(": {:?}", e));
        eprintln!();
        std::process::exit(1);
    }
}
