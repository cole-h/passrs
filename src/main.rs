// FIXME: ensure all/most code is DRY -- Don't Repeat Yourself
// FIXME: document EVERYTHING -- all functions, structs, etc
//   https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments
//   https://doc.rust-lang.org/rustdoc/index.html
//   https://doc.rust-lang.org/stable/reference/comments.html#doc-comments
//   https://doc.rust-lang.org/rust-by-example/meta/doc.html

use passrs::cli;

fn main() {
    if let Err(err) = cli::opt() {
        // eprintln!("{:?}", err.backtrace()); // this displays the backtrace
        //                                     // (which requires a nightly compiler)

        eprint!("{}", err);
        err.chain().skip(1).for_each(|e| eprint!(": {}", e));
        eprintln!();
        std::process::exit(1);
    }
}
