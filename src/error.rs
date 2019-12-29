use failure::Fail;

#[derive(Debug, Fail)]
pub(crate) enum PassrsError {
    #[fail(display = "No private key found")]
    NoPrivateKeyFound,
    #[fail(display = "Path exists: {}", _0)]
    PathExists(String),
    #[fail(display = "Failed to init git repo")]
    FailedToInitGitRepo,
    #[fail(display = "Failed to open git repo")]
    FailedToOpenGitRepo,
    #[fail(display = "Failed to create directories")]
    FailedToCreateDirectories,
    #[fail(display = "No matches found for search '{}'", _0)]
    NoMatchesFound(String),
    #[fail(display = "The entered passwords do not match.")]
    PasswordsDontMatch,
    #[fail(display = "Hashes don't match: {} vs {}", _0, _1)]
    HashMismatch(String, String),
    #[fail(display = "User aborted")]
    UserAbort,
    #[cfg(feature = "otp")]
    #[fail(display = "URI was not in valid Key Uri Format.\n\
                      See https://github.com/google/google-authenticator/wiki/Key-Uri-Format for more information.")]
    InvalidKeyUri,
    #[cfg(feature = "otp")]
    #[fail(display = "Invalid hash function: {}", _0)]
    InvalidHashFunction(String),
    #[fail(display = "No signing key found")]
    NoSigningKeyFound,
    #[fail(display = "Path does not exist: {}", _0)]
    PathDoesntExist(String),
    #[fail(display = "Sneaky path: {}", _0)]
    SneakyPath(String),
    #[fail(display = "Store does not exist")]
    StoreDoesntExist,
}

// NOTE: external errors
// #[fail(display = "{}", _0)]
// GpgError(#[fail(cause)] gpgme::Error),
// #[fail(display = "{}", _0)]
// GitError(#[fail(cause)] git2::Error),
// #[fail(display = "{}", _0)]
// Io(std::io::Error)
