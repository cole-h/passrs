// FIXME: replace all print* calls with actual logging
// FIXME: some funcs take Path, some take &str, some take String, and some take
// PathBuf... wtf bro
// FIXME: deal with all unwraps, unless I can guarantee that it won't panic
// FIXME: ensure all/most code is DRY -- Don't Repeat Yourself
// FIXME: document EVERYTHING -- all functions, structs, etc
//   https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments
//   https://doc.rust-lang.org/rustdoc/index.html
//   https://doc.rust-lang.org/stable/reference/comments.html#doc-comments
//   https://doc.rust-lang.org/rust-by-example/meta/doc.html

mod cli;
mod clipboard;
mod consts;
mod error;
mod event;
mod subcmds;
mod tree;
mod ui;
mod util;

fn main() -> failure::Fallible<()> {
    match cli::opt() {
        Ok(_) => {}
        Err(e) => {
            // Gracefully handle errors so backtraces only happen on legitimate
            // panics.
            eprintln!("{}", e);
            // eprintln!("{:?}", e); // this displays the backtrace
            std::process::exit(1);
        }
    }
    // cli::opt()?;

    Ok(())
}

// TODO: every subcommand should use the following scaffolding before doing
// anything else:
// 1. ensure directories are created
// 2. canonicalize paths
// 3. check for sneaky paths -- what does this entail with Rust?
//
// 99. commit INSIDE THE SUBCMD
