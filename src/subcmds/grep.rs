use std::io;

use anyhow::{Context, Result};
use grep_printer::{ColorSpecs, StandardBuilder};
use grep_regex::RegexMatcher;
use grep_searcher::{BinaryDetection, SearcherBuilder};
use termcolor::{BufferedStandardStream, ColorChoice, StandardStream};
use termion::style;
use walkdir::WalkDir;
use zeroize::Zeroize;

use crate::consts::{HOME, PASSWORD_STORE_DIR, PASSWORD_STORE_LEN};
use crate::util;

// Takes ~40 seconds to search the entirety of my ~400 file store
pub fn grep(search: String) -> Result<()> {
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
        // easily. We use `termcolor` because I want colors for matches, so we
        // can get `StandardStream` and `BufferedStandardStream` for free.
        .build({
            let color = if termion::is_tty(&io::stdout()) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            };
            if termion::is_tty(&io::stdout()) {
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
            .with_context(|| "Entry did not contain a valid path")?;
        if !path.ends_with(".gpg") {
            continue;
        }

        // I want path[..separator] to include the final slash, so add 1
        let separator = path
            .rfind('/')
            .with_context(|| "Path did not contain a folder")?
            + 1;
        let pre = path[*PASSWORD_STORE_LEN..separator].replace(&*HOME, "~");
        // We guarantee all paths end in .gpg by this point, so we can cut it
        // off without a problem (famous last words)
        let file = &path[separator..path.len() - 4];
        let mut contents = util::decrypt_file_into_bytes(path)?;
        let formatted_path = format!("{}{bold}{}", pre, file, bold = style::Bold);

        searcher.search_slice(
            &matcher,
            &contents,
            printer.sink_with_path(&matcher, &formatted_path),
        )?;
        contents.zeroize();
    }

    Ok(())
}

// TODO: properly attribute
/// ref: https://docs.rs/grep-cli/0.1.3/src/grep_cli/wtr.rs.html
enum StandardStreamKind {
    LineBuffered(termcolor::StandardStream),
    BlockBuffered(termcolor::BufferedStandardStream),
}

impl io::Write for StandardStreamKind {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            StandardStreamKind::LineBuffered(ref mut w) => w.write(buf),
            StandardStreamKind::BlockBuffered(ref mut w) => w.write(buf),
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
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
    fn set_color(&mut self, spec: &termcolor::ColorSpec) -> io::Result<()> {
        match self {
            StandardStreamKind::LineBuffered(ref mut w) => w.set_color(spec),
            StandardStreamKind::BlockBuffered(ref mut w) => w.set_color(spec),
        }
    }

    #[inline]
    fn reset(&mut self) -> io::Result<()> {
        match self {
            StandardStreamKind::LineBuffered(ref mut w) => w.reset(),
            StandardStreamKind::BlockBuffered(ref mut w) => w.reset(),
        }
    }
}
