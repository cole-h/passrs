//! Errors via [thiserror]
//!
//! # error
//!
//! This module contains all possible error variants related to the operation of
//! this program.
//!
//! [thiserror]: https://docs.rs/thiserror

use termion::{color, style};
use thiserror::Error;

const RED: color::Fg<color::Red> = color::Fg(color::Red);
const RESET: style::Reset = style::Reset;

#[derive(Debug, Error)]
pub(crate) enum PassrsError {
    #[error("{RED}Error: No private key found{RESET}")]
    NoPrivateKeyFound,
    #[error("{RED}Error: No matches found for '{0}'{RESET}")]
    NoMatchesFound(String),
    #[error("{RED}Error: The entered secrets do not match.{RESET}")]
    SecretsDontMatch,
    #[error("{RED}Error: Hashes don't match{RESET}")]
    HashMismatch,
    #[error("{RED}Error: User aborted{RESET}")]
    UserAbort,
    #[cfg(feature = "otp")]
    #[error("{RED}Error: URI was not in valid Key Uri Format.{RESET}\n\
             See https://github.com/google/google-authenticator/wiki/Key-Uri-Format for more information.")]
    InvalidKeyUri,
    #[cfg(feature = "otp")]
    #[error("{RED}Error: Invalid hash function: '{0}'{RESET}")]
    InvalidHashAlgorithm(String),
    #[cfg(feature = "otp")]
    #[error("{RED}Error: No URIs found in entry '{0}'{RESET}")]
    NoUriFound(String),
    #[error("{RED}Error: No signing key found{RESET}")]
    NoSigningKeyFound,
    #[error("{RED}Error: Path '{0}' does not exist{RESET}")]
    PathDoesntExist(String),
    #[error("{RED}Error: Sneaky path '{0}'{RESET}")]
    SneakyPath(String),
    #[error(
        "{RED}Error: Store does not exist{RESET}\n\
             You must run `pass init gpg-id` before you can \
             use the password store"
    )]
    StoreDoesntExist,
    #[error("{RED}Error: '{0}' is not in the password store{RESET}")]
    NotInStore(String),
    #[error("{RED}Error: Source is destination{RESET}")]
    SourceIsDestination,
    #[error("{RED}Error: '{0}' is a directory{RESET}")]
    PathIsDir(String),
    #[error("Contents unchanged")] // don't color this one because it's just information
    ContentsUnchanged,
    #[error("{RED}Error: Failed to get contents of clipboard{RESET}")]
    PasteFailed,
    #[error("{RED}Error: No `.gpg-id` was found in '{0}'{RESET}")]
    NoGpgIdFile(String),
    #[error("{RED}Error: Failed to set contents of clipboard{RESET}")]
    ClipFailed,
    #[error("{RED}Error: stdout was not a tty{RESET}")]
    StdoutNotTty,
}
