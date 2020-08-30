fn main() {
    let ver = env!("CARGO_PKG_VERSION");
    let rev_env = std::env::var("PASSRS_REV");
    let rev_cmd = std::process::Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|r| String::from_utf8(r.stdout).ok());

    let version = match (rev_env, rev_cmd) {
        (Ok(rev), _) => format!("{} ({})", ver, rev.trim()),
        (_, Some(rev)) => format!("{} ({})", ver, rev.trim()),
        _ => format!("{} (unknown)", ver),
    };

    println!("cargo:rustc-env=PASSRS_VERSION={}", version);
}
