// FIXME: replace all print* calls with actual logging
#![forbid(unsafe_code)]

mod cli;
mod clipboard;
mod consts;
mod error;
mod event;
mod subcmds;
mod tree;
mod ui;
mod util;

fn main() -> Result<(), failure::Error> {
    // FIXME: use to gracefuly handle errors
    // match cli::opt() {
    //     Ok(_) => {}
    //     Err(e) => {
    //         eprintln!("{:?}", e);
    //         std::process::exit(1);
    //     }
    // }
    // cli::opt()?;

    let tree = tree::tree("/tmp/passrstest")?;

    dbg!(&tree);
    let trees = tree::find("test", tree)?;
    dbg!(&trees[0]);
    for (idx, tree) in trees.iter().enumerate() {
        println!("{}: {}", idx, tree);
    }

    // println!("{}", tree);

    // let matches = util::search_entries("reddit")?;
    // match ui::display_matches(&matches) {
    //     Ok(_) => {}
    //     Err(e) => {
    //         eprintln!("{}", e);
    //         std::process::exit(1);
    //     }
    // }
    // util::encrypt_bytes_into_file("test.gpg", &[b'l', b'm', b'a', b'o'])?;

    Ok(())
}
