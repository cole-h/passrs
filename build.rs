fn main() {
    let ver = env!("CARGO_PKG_VERSION");
    let rev = std::process::Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|r| String::from_utf8(r.stdout).ok());

    let version = match rev {
        Some(rev) if !rev.is_empty() => format!("{} ({})", ver, rev.trim()),
        _ => format!("{} (unknown)", ver),
    };

    println!("cargo:rustc-env=PASSRS_VERSION={}", version);
}
