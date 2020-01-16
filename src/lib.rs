pub mod cli;
pub mod clipboard;
pub mod consts;
pub mod error;
#[cfg(feature = "otp")]
pub mod otp;
pub mod subcmds;
pub mod tree;
pub mod ui;
pub mod util;

use cli::Flags;
use error::PassrsError;
