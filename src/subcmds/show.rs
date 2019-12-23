use std::io::Write;
use std::process::{Command, Stdio};

use ring::digest;

pub fn show(clip: bool, pass_name: String) {
    let _ = pass_name;
    let password = "";
    // let password = get_decrypted_file(default_path, pass_name)

    if clip {
        // spawn subcommand for 45 seconds
        let hash = hex::encode(digest::digest(&digest::SHA256, password.as_bytes()));

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
            .args(vec!["unclip", "45"])
            .env("PASSRS_UNCLIP_HASH", hash)
            .spawn()
            .unwrap();
    }

    // TODO: show password
}
