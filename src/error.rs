//! Errors
//!
//! # error
//!
//! This module contains all possible error variants related to the operation of
//! this program.

use std::error::Error;
use std::fmt;

use termion::{color, style};

const RED: color::Fg<color::Red> = color::Fg(color::Red);
const RESET: style::Reset = style::Reset;

pub type Result<T, E = Box<dyn Error + Send + Sync + 'static>> = core::result::Result<T, E>;

#[derive(Debug)]
pub(crate) enum PassrsError {
    NoPrivateKeyFound,
    NoMatchesFound(String),
    SecretsDontMatch,
    HashMismatch,
    UserAbort,
    InvalidKeyUri,
    InvalidHashAlgorithm(String),
    NoUriFound(String),
    NoSigningKeyFound,
    PathDoesntExist(String),
    SneakyPath(String),
    StoreDoesntExist,
    NotInStore(String),
    SourceIsDestination,
    PathIsDir(String),
    ContentsUnchanged,
    PasteFailed,
    NoGpgIdFile(String),
    ClipFailed,
    StdoutNotTty,
    Other(String),
}

impl fmt::Display for PassrsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PassrsError::*;

        match self {
            NoPrivateKeyFound => write!(
                f,
                "{RED}Error: No private key found{RESET}",
                RED = RED,
                RESET = RESET
            ),
            NoMatchesFound(s) => write!(
                f,
                "{RED}Error: No matches found for '{}'{RESET}",
                s,
                RED = RED,
                RESET = RESET
            ),
            SecretsDontMatch => write!(
                f,
                "{RED}Error: The entered secrets do not match.{RESET}",
                RED = RED,
                RESET = RESET
            ),
            HashMismatch => write!(
                f,
                "{RED}Error: Hashes don't match{RESET}",
                RED = RED,
                RESET = RESET
            ),
            UserAbort => write!(
                f,
                "{RED}Error: User aborted{RESET}",
                RED = RED,
                RESET = RESET
            ),
            InvalidKeyUri => write!(
                f,
                "{RED}Error: URI was not in valid Key Uri Format.{RESET}\n\
                 See https://github.com/google/google-authenticator/wiki/Key-Uri-Format for more information.",
                RED=RED,
                RESET=RESET
            ),
            InvalidHashAlgorithm(s) => write!(
                f,
                "{RED}Error: Invalid hash function: '{}'{RESET}",
                s,
                RED = RED,
                RESET = RESET
            ),
            NoUriFound(s) => write!(
                f,
                "{RED}Error: No URIs found in entry '{}'{RESET}",
                s,
                RED = RED,
                RESET = RESET
            ),
            NoSigningKeyFound => write!(
                f,
                "{RED}Error: No signing key found{RESET}",
                RED = RED,
                RESET = RESET
            ),
            PathDoesntExist(s) => write!(
                f,
                "{RED}Error: Path '{}' does not exist{RESET}",
                s,
                RED = RED,
                RESET = RESET
            ),
            SneakyPath(s) => write!(
                f,
                "{RED}Error: Sneaky path '{}'{RESET}",
                s,
                RED = RED,
                RESET = RESET
            ),
            StoreDoesntExist => write!(
                f,
                "{RED}Error: Store does not exist{RESET}\n\
                    You must run `pass init gpg-id` before you can \
                    use the password store",
                RED = RED,
                RESET = RESET
            ),
            NotInStore(s) => write!(
                f,
                "{RED}Error: '{}' is not in the password store{RESET}",
                s,
                RED = RED,
                RESET = RESET
            ),
            SourceIsDestination => write!(
                f,
                "{RED}Error: Source is destination{RESET}",
                RED = RED,
                RESET = RESET
            ),
            PathIsDir(s) => write!(
                f,
                "{RED}Error: '{}' is a directory{RESET}",
                s,
                RED = RED,
                RESET = RESET
            ),
            ContentsUnchanged => write!(f, "Contents unchanged"), // don't color this one because it's just information
            PasteFailed => write!(
                f,
                "{RED}Error: Failed to get contents of clipboard{RESET}",
                RED = RED,
                RESET = RESET
            ),
            NoGpgIdFile(s) => write!(
                f,
                "{RED}Error: No `.gpg-id` was found in '{}'{RESET}",
                s,
                RED = RED,
                RESET = RESET
            ),
            ClipFailed => write!(
                f,
                "{RED}Error: Failed to set contents of clipboard{RESET}",
                RED = RED,
                RESET = RESET
            ),
            StdoutNotTty => write!(
                f,
                "{RED}Error: stdout was not a tty{RESET}",
                RED = RED,
                RESET = RESET
            ),
            Other(s) => write!(
                f,
                "{RED}Error: {}{RESET}",
                s,
                RED=RED,
                RESET=RESET,
            )
        }
    }
}

impl Error for PassrsError {}

impl From<String> for PassrsError {
    fn from(error: String) -> Self {
        PassrsError::Other(error)
    }
}
