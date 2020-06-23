//! The `passrs` binary

use std::io::{self, Write};

fn main() -> io::Result<()> {
    if let Err(err) = passrs::cli::opt() {
        write!(io::stderr(), "{}", err)?;

        // TODO: Either impl own "chaining", or say screw it
        // err.chain()
        //     .skip(1)
        //     .for_each(|e| write!(io::stderr(), ": {}", e).expect("Failed to write to stderr"));

        writeln!(io::stderr())?;

        std::process::exit(1);
    }

    Ok(())
}
