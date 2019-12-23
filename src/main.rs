use std::error::Error;

mod cli;
pub(crate) mod consts;
pub(crate) mod error;
pub(crate) mod subcmds;
pub(crate) mod utils;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    cli::opt();
}
