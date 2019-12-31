use failure::Fail;

#[derive(Debug, Fail)]
pub(crate) enum PassrsError {
    #[fail(display = "Error: No private key found")]
    NoPrivateKeyFound,
    #[fail(display = "Error: Path {} exists", _0)]
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
    #[fail(display = "Error: Hashes don't match: {} vs {}", _0, _1)]
    HashMismatch(String, String),
    #[fail(display = "Error: User aborted")]
    UserAbort,
    #[cfg(feature = "otp")]
    #[fail(display = "Error: URI was not in valid Key Uri Format.\n\
                      See https://github.com/google/google-authenticator/wiki/Key-Uri-Format for more information.")]
    InvalidKeyUri,
    #[cfg(feature = "otp")]
    #[fail(display = "Error: Invalid hash function: {}", _0)]
    InvalidHashFunction(String),
    #[fail(display = "Error: No signing key found")]
    NoSigningKeyFound,
    #[fail(display = "Error: Path {} does not exist", _0)]
    PathDoesntExist(String),
    #[fail(display = "Error: Sneaky path {}", _0)]
    SneakyPath(String),
    #[fail(display = "Error: Store does not exist")]
    StoreDoesntExist,
    #[fail(display = "Error: {} is not in the password store", _0)]
    NotInStore(String),
    #[fail(display = "Error: Source is destination")]
    SourceIsDestination,
    #[fail(display = "Error: {} is a directory", _0)]
    PathIsDir(String),
    #[fail(display = "Error: Key {} is already the signing key", _0)]
    SameKey(String),
    #[fail(display = "Contents unchanged")]
    ContentsUnchanged,
}

// NOTE: external errors
// #[fail(display = "{}", _0)]
// GpgError(#[fail(cause)] gpgme::Error),
// #[fail(display = "{}", _0)]
// GitError(#[fail(cause)] git2::Error),
// #[fail(display = "{}", _0)]
// Io(std::io::Error)
