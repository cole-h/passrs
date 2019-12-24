// FIXME: replace all print* calls with actual logging

mod cli;
mod consts;
mod error;
#[cfg(feature = "tui")]
mod event;
mod subcmds;
#[cfg(feature = "tui")]
mod ui;
mod utils;

// use std::error::Error;
// pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
// pub type Result<T> = ::std::result::Result<T, failure::Error>;

fn main() -> Result<(), failure::Error> {
    // match cli::opt() {
    //     Ok(_) => Ok(()),
    //     Err(e) => {
    //         println!("{}", e);
    //         std::process::exit(1);
    //     }
    // }
    // let _ = utils::search_entries("sep")?;
    ui::display_matches("sep")?;
    // ui::select().unwrap();
    Ok(())
}
