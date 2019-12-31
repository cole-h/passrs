use failure::{err_msg, Fallible};
use grep_printer::{ColorSpecs, StandardBuilder};
use grep_regex::RegexMatcher;
use grep_searcher::{BinaryDetection, SearcherBuilder};
use termcolor::{BufferedStandardStream, ColorChoice, StandardStream};
use termion::style;
use walkdir::WalkDir;

use crate::consts::{HOME, PASSWORD_STORE_DIR};
use crate::util;

// Takes ~40 seconds to search the entirety of my ~400 file store
pub fn grep(search: String) -> Fallible<()> {
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();
    let matcher = RegexMatcher::new_line_matcher(&search)?;
    let mut printer = StandardBuilder::new()
        .color_specs(ColorSpecs::new(&[
            "path:fg:blue".parse()?,
            "line:fg:green".parse()?,
            "match:fg:red".parse()?,
            "match:style:bold".parse()?,
        ]))
        .heading(true)
        // Shorter and more concise way to do this is to use grep_cli, but we
        // only use it for 2 things, and it pulls in 3 extra dependencies. Both
        // cli::stdout() and cli::is_tty_stdout() can be implemented here fairly
        // easily. `atty` is already a dependency of `clap`, so we get that for
        // free. We use `termcolor` because I want colors for matches, so we get
        // `StandardStream` and `BufferedStandardStream` for free.
        .build({
            let color = if atty::is(atty::Stream::Stdout) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            };
            if atty::is(atty::Stream::Stdout) {
                let out = StandardStream::stdout(color);
                StandardStreamKind::LineBuffered(out)
            } else {
                let out = BufferedStandardStream::stdout(color);
                StandardStreamKind::BlockBuffered(out)
            }
        });

    for dirent in WalkDir::new(&*PASSWORD_STORE_DIR) {
        let entry = dirent?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry
            .path()
            .to_str()
            .ok_or_else(|| err_msg("Entry did not contain a valid path"))?;
        if !path.ends_with(".gpg") {
            continue;
        }

        let separator = path
            .rfind('/')
            .ok_or_else(|| err_msg("Path did not contain a folder"))?
            + 1;
        let pre = path[PASSWORD_STORE_DIR.len()..separator].replace(&*HOME, "~");
        // We guarantee all paths end in .gpg by this point, so we can cut it
        // off without a problem (famous last words)
        let file = &path[separator..path.len() - 4];
        let contents = util::decrypt_file_into_bytes(path)?;
        let formatted_path = format!("{}{bold}{}", pre, file, bold = style::Bold);

        searcher.search_slice(
            &matcher,
            &contents,
            printer.sink_with_path(&matcher, &formatted_path),
        )?;
    }

    Ok(())
}

// TODO: properly attribute
/// ref: https://docs.rs/grep-cli/0.1.3/src/grep_cli/wtr.rs.html
enum StandardStreamKind {
    LineBuffered(termcolor::StandardStream),
    BlockBuffered(termcolor::BufferedStandardStream),
}

impl std::io::Write for StandardStreamKind {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            StandardStreamKind::LineBuffered(ref mut w) => w.write(buf),
            StandardStreamKind::BlockBuffered(ref mut w) => w.write(buf),
        }
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            StandardStreamKind::LineBuffered(ref mut w) => w.flush(),
            StandardStreamKind::BlockBuffered(ref mut w) => w.flush(),
        }
    }
}

impl termcolor::WriteColor for StandardStreamKind {
    #[inline]
    fn supports_color(&self) -> bool {
        match self {
            StandardStreamKind::LineBuffered(ref w) => w.supports_color(),
            StandardStreamKind::BlockBuffered(ref w) => w.supports_color(),
        }
    }

    #[inline]
    fn set_color(&mut self, spec: &termcolor::ColorSpec) -> std::io::Result<()> {
        match self {
            StandardStreamKind::LineBuffered(ref mut w) => w.set_color(spec),
            StandardStreamKind::BlockBuffered(ref mut w) => w.set_color(spec),
        }
    }

    #[inline]
    fn reset(&mut self) -> std::io::Result<()> {
        match self {
            StandardStreamKind::LineBuffered(ref mut w) => w.reset(),
            StandardStreamKind::BlockBuffered(ref mut w) => w.reset(),
        }
    }
}
