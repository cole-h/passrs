//! The `passrs` binary
// TODO: check out sequoia-openpgp: https://gitlab.com/sequoia-pgp/sequoia

use std::io;
use std::io::Write;

fn main() -> io::Result<()> {
    if let Err(err) = passrs::cli::opt() {
        write!(io::stderr(), "{}", err)?;
        err.chain()
            .skip(1)
            .for_each(|e| write!(io::stderr(), ": {}", e).expect("Failed to write to stderr"));
        writeln!(io::stderr())?;

        std::process::exit(1);
    }

    Ok(())
}
