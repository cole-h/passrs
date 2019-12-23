use std::error::Error;
use std::fmt;

// TODO: better error handling
#[derive(Debug)]
pub(crate) enum PassrsError {
    NoPrivateKeyFound,
    PathExists,
    FailedToInitGitRepo,
    FailedToCreateDirectories,
}

impl fmt::Display for PassrsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PassrsError {}
