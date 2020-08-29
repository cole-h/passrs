use std::env;

fn main() {
    let ver = env!("CARGO_PKG_VERSION");
    let rev = env::var("PASSRS_REV");

    let version = match rev {
        Ok(rev) => format!("{} ({})", ver, rev.trim()),
        _ => format!("{} (unknown)", ver),
    };

    println!("cargo:rustc-env=PASSRS_VERSION={}", version);
}
