//! The `passrs` binary

fn main() {
    if let Err(err) = pass_rs::cli::opt() {
        eprint!("{}", err);
        err.chain().skip(1).for_each(|e| eprint!(": {}", e));
        eprintln!();

        std::process::exit(1);
    }
}
