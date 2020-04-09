//! The `passrs` binary is a reimplementation of [`pass`] in Rust. Most of the
//! functionality has been ported, but not all.
//!
//! ## Present functionality
//! * initialize a new password store: `passrs init <YOUR_GPG_ID>`
//! * list of all secrets in the store inspired by the original implementation
//! using `tree`: `passrs ls` or just `passrs`
//! * find all secrets that match the specified name: `passrs find <entry-name>`
//!   * For now, this prints the full path of the entry and does not display it
//!   as a tree
//! * grep for secrets that match a string when decrypted: `passrs grep
//! <search-string>`
//! * insert a new secret: `passrs insert <entry-name>`
//! * edit a secret using the `$EDITOR` environment variable: `passrs edit
//! <entry-name>`
//! * generate a new secret with a specified length (defaults to 24): `passrs
//! generate <entry-name> [pass-length]`
//! * remove an entry from the store: `passrs rm <entry-name>`
//! * move an entry to a new location: `passrs mv <old-path> <new-path>`
//! * copy an entry to a new location: `passrs cp <old-path> <new-path>`
//! * execute arbitrary `git` commands: `passrs git [git-command-args]`
//! * manage OTP tokens
//!   * append an OTP secret to the specified entry: `passrs otp append
//!   <entry-name>`
//!   * generate a TOTP code from the specified entry: `passrs otp code
//!   <entry-name>`
//!   * insert an OTP secret to the specified entry: `passrs otp insert
//!   <entry-name>`
//!   * print the key URI of the specified entry: `passrs otp uri <entry-name>`
//!   * validate a URI string for adherence to the [Key Uri Format]: `passrs otp
//!   validate <uri>`
//! * print shell completion information to stdout: `passrs complete bash`
//!
//! ## Missing functionality
//! * support for certain environment variables:
//!   * `PASSWORD_STORE_ENABLE_EXTENSIONS`
//!   * `PASSWORD_STORE_EXTENSIONS_DIR`
//!   * `PASSWORD_STORE_GPG_OPTS`
//!   * `GREPOPTIONS`
//! * deinitialization of the password store
//! * entries found with `passrs find` being displayed as a tree
//! * probably more
//!
//! [`pass`]: https://passwordstore.org
//! [gpgme]: https://docs.rs/gpgme
//! [Key Uri Format]: https://github.com/google/google-authenticator/wiki/Key-Uri-Format

#[doc(hidden)]
pub mod cli;
pub mod clipboard;
pub mod consts;
pub mod error;
#[cfg(feature = "otp")]
pub mod otp;
#[doc(hidden)]
pub mod subcmds;
pub mod tree;
pub mod ui;
pub mod util;

use cli::Flags;
use error::PassrsError;
