use std::io;

use grep_printer::{ColorSpecs, StandardBuilder};
use grep_regex::RegexMatcher;
use grep_searcher::{BinaryDetection, SearcherBuilder};
use termcolor::{BufferedStandardStream, ColorChoice, StandardStream};
use termion::style;
use walkdir::WalkDir;

use crate::consts::{PASSWORD_STORE_DIR, STORE_LEN};
use crate::util;
use crate::Result;

// Takes ~40 seconds to search the entirety of my ~400 file store
pub(crate) fn grep(search: String) -> Result<()> {
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
            if termion::is_tty(&io::stdout()) {
                let out = StandardStream::stdout(ColorChoice::Auto);
                stream::StandardStreamKind::LineBuffered(out)
            } else {
                let out = BufferedStandardStream::stdout(ColorChoice::Never);
                stream::StandardStreamKind::BlockBuffered(out)
            }
        });

    for dirent in WalkDir::new(&*PASSWORD_STORE_DIR) {
        let entry = dirent?;
        let path = entry
            .path()
            .to_str()
            .ok_or("Entry did not contain a valid path")?;

        if !entry.file_type().is_file() || !path.ends_with(".gpg") {
            continue;
        }

        // I want path[..separator] to include the final slash, so add 1
        let separator = path.rfind('/').ok_or("Path did not contain a folder")? + 1;
        let pre = &path[*STORE_LEN..separator];
        // We guarantee all paths end in .gpg by this point, so we can cut it
        // off without a problem (famous last words)
        let file = &path[separator..path.rfind(".gpg").unwrap()];
        let contents = util::decrypt_file_into_bytes(path)?;
        let formatted_path = format!(
            "{bold}{}{nobold}{}",
            pre,
            file,
            bold = style::Bold,
            nobold = style::NoBold
        );

        searcher.search_slice(
            &matcher,
            &contents,
            printer.sink_with_path(&matcher, &formatted_path),
        )?;
    }

    Ok(())
}

mod stream {
    // https://github.com/BurntSushi/ripgrep/blob/4846d63539690047fa58ec582d94bcba16da1c09/grep-cli/src/wtr.rs#L66
    // Copyright (c) 2015 Andrew Gallant
    //
    // Permission is hereby granted, free of charge, to any person obtaining a copy
    // of this software and associated documentation files (the "Software"), to deal
    // in the Software without restriction, including without limitation the rights
    // to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    // copies of the Software, and to permit persons to whom the Software is
    // furnished to do so, subject to the following conditions:
    //
    // The above copyright notice and this permission notice shall be included in
    // all copies or substantial portions of the Software.
    //
    // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    // IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    // OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
    // THE SOFTWARE.
    use std::io;

    pub enum StandardStreamKind {
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
}
