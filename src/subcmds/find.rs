use std::io::{self, Write};

use termion::color;
use termion::style;

use crate::consts::STORE_LEN;
use crate::util;
use crate::{PassrsError, Result};

pub(crate) fn find(name: String) -> Result<()> {
    let matches = util::find_matches(&name)?;

    // FIXME: For now, just following `gopass`s lead: straight-up print the
    // matches to stdout.
    // Printing trees is a pain in the ass. Filtering them is even worse.

    if !matches.is_empty() {
        for matched in &matches {
            let separator = matched.rfind('/').ok_or("Path did not contain a folder")? + 1;
            let pre = &matched[*STORE_LEN..separator];
            let file = &matched[separator..matched.rfind(".gpg").unwrap()];
            let formatted_path = format!(
                "{blue}{bold}{}{nobold}{}{reset}",
                pre,
                file,
                blue = color::Fg(color::Blue),
                bold = style::Bold,
                nobold = style::NoBold,
                reset = style::Reset,
            );

            writeln!(io::stdout(), "{}", formatted_path)?;
        }
    } else {
        return Err(PassrsError::NoMatchesFound(name).into());
    }

    Ok(())
}
