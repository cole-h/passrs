use termion::color;
use termion::style;
use thiserror::Error;

// TODO: make ContextExt and make with_context show line numbers
//   https://github.com/dtolnay/anyhow/issues/22
#[derive(Debug, Error)]
pub(crate) enum PassrsError {
    #[error("{}Error: No private key found{}", color::Fg(color::Red), style::Reset)]
    NoPrivateKeyFound,
    #[error(
        "{}Error: No matches found for search '{0}'{}",
        color::Fg(color::Red),
        style::Reset
    )]
    NoMatchesFound(String),
    #[error(
        "{}Error: The entered secrets do not match.{}",
        color::Fg(color::Red),
        style::Reset
    )]
    SecretsDontMatch,
    #[error("{}Error: Hashes don't match{}", color::Fg(color::Red), style::Reset)]
    HashMismatch,
    #[error("{}Error: User aborted{}", color::Fg(color::Red), style::Reset)]
    UserAbort,
    #[cfg(feature = "otp")]
    #[error("Error: URI was not in valid Key Uri Format.\n\
             See https://github.com/google/google-authenticator/wiki/Key-Uri-Format for more information.")]
    InvalidKeyUri,
    #[cfg(feature = "otp")]
    #[error(
        "{}Error: Invalid hash function: '{0}'{}",
        color::Fg(color::Red),
        style::Reset
    )]
    InvalidHashAlgorithm(String),
    #[error("{}Error: No signing key found{}", color::Fg(color::Red), style::Reset)]
    NoSigningKeyFound,
    #[error(
        "{}Error: Path '{0}' does not exist{}",
        color::Fg(color::Red),
        style::Reset
    )]
    PathDoesntExist(String),
    #[error("{}Error: Sneaky path '{0}'{}", color::Fg(color::Red), style::Reset)]
    SneakyPath(String),
    #[error("{}Error: Store does not exist{}", color::Fg(color::Red), style::Reset)]
    StoreDoesntExist,
    #[error(
        "{}Error: '{0}' is not in the password store{}",
        color::Fg(color::Red),
        style::Reset
    )]
    NotInStore(String),
    #[error(
        "{}Error: Source is destination{}",
        color::Fg(color::Red),
        style::Reset
    )]
    SourceIsDestination,
    #[error("{}Error: '{0}' is a directory{}", color::Fg(color::Red), style::Reset)]
    PathIsDir(String),
    #[error("Contents unchanged")]
    ContentsUnchanged,
    #[error(
        "{}Error: Failed to get contents of clipboard{}",
        color::Fg(color::Red),
        style::Reset
    )]
    PasteFailed,
    #[error(
        "{}Error: No `.gpg-id` was found in '{0}'{}",
        color::Fg(color::Red),
        style::Reset
    )]
    NoGpgIdFile(String),
    #[error(
        "{}Error: Signature for '{0}' does not exist{}",
        color::Fg(color::Red),
        style::Reset
    )]
    MissingSignature(String),
    #[error(
        "{}Error: Signature for '{0}' does not match{}",
        color::Fg(color::Red),
        style::Reset
    )]
    BadSignature(String),
    #[error(
        "{}Error: Failed to set contents of clipboard{}",
        color::Fg(color::Red),
        style::Reset
    )]
    ClipFailed,
}
