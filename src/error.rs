use failure::Fail;

// TODO: better error handling
#[derive(Debug, Fail)]
pub(crate) enum PassrsError {
    #[fail(display = "No private key found")]
    NoPrivateKeyFound,
    #[fail(display = "Path exists")]
    PathExists,
    #[fail(display = "Failed to init git repo")]
    FailedToInitGitRepo,
    #[fail(display = "Failed to create directories")]
    FailedToCreateDirectories,
    #[fail(display = "URI was not in valid Key Uri Format.\n\
                      See https://github.com/google/google-authenticator/wiki/Key-Uri-Format for more information.")]
    InvalidKeyUri,
    #[fail(display = "No matches found for search {}", _0)]
    NoMatchesFound(String),
    // #[fail(display = "{}", _0)]
    // GpgError(#[fail(cause)] gpgme::Error),
    // #[fail(display = "{}", _0)]
    // GitError(#[fail(cause)] git2::Error),
    // #[fail(display = "{}", _0)]
    // Io(::std::io::Error),
}
