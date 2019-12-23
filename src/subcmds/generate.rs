use std::io::Write;
use std::process::{Command, Stdio};

use rand::Rng;
use ring::digest;

use crate::consts::{ALPHA_LOWER, ALPHA_UPPER, DIGITS, SPECIAL};

pub fn generate(
    rng: &mut impl Rng,
    no_symbols: bool,
    clip: bool,
    in_place: bool,
    force: bool,
    pass_name: String,
    pass_length: u32,
) -> Option<String> {
    let set = if no_symbols {
        [DIGITS, ALPHA_LOWER, ALPHA_UPPER].concat()
    } else {
        [DIGITS, ALPHA_LOWER, ALPHA_UPPER, SPECIAL].concat()
    };
    let len = pass_length;
    let mut password_bytes: Vec<u8> = Vec::new();

    for _ in 0..=len {
        let idx = rng.gen_range(0, set.len());
        password_bytes.push(set[idx]);
    }
    // TODO: is it worth it to use unchecked?
    // we are 100% sure the password is valid utf8 because we specify the valid characters
    //   in `set`
    let password = unsafe { std::str::from_utf8_unchecked(&password_bytes) }.to_owned();
    // TODO: remove me
    #[cfg(debug_assertions)]
    println!("{}", password);

    if in_place {
        // 1. decrypt file
        // 2. read into vec of lines
        // 3. modify first line
        // 4. serialize to be re-encrypted
        // --OR--
        // 1. decrypt file
        // 2. find a way to overwrite first line (until CRLF/LF?)
    }

    if clip {
        let hash = hex::encode(digest::digest(&digest::SHA256, password.as_bytes()));
        let args = vec!["unclip", if force { "--force" } else { "" }, "45"];
        let args = args.iter().filter(|&&x| x != "").collect::<Vec<_>>();

        Command::new("wl-copy").arg("--clear").status().unwrap();
        // TODO: genericize over wl-copy, xclip, pbcopy, etc
        //   ref: https://github.com/atotto/clipboard/blob/e9e854e353882a018e9dc587e3757a8822958941/clipboard_unix.go
        Command::new("wl-copy")
            .arg("--trim-newline")
            .stdin(Stdio::piped())
            .spawn()
            .unwrap()
            .stdin
            .unwrap()
            .write_all(password.as_bytes())
            .unwrap();

        // otherwise, the process doesn't live long enough
        std::thread::sleep(std::time::Duration::from_millis(50));

        Command::new(std::env::current_exe().unwrap())
            .args(args)
            .env("PASSRS_UNCLIP_HASH", hash)
            .spawn()
            .unwrap();
    }

    Some(format!("Save generated secret to {}", pass_name))
}
