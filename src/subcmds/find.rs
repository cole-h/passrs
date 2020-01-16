use anyhow::{Context, Result};
use termion::color;
use termion::style;

use crate::consts::STORE_LEN;
use crate::util;
use crate::PassrsError;

pub fn find(name: String) -> Result<()> {
    let matches = util::find_target_single(&name)?;

    // FIXME: For now, just following `gopass`s lead: straight-up print the
    // matches to stdout.
    // Printing trees is a pain in the ass. Filtering them is even worse.

    if !matches.is_empty() {
        for matched in &matches {
            let separator = matched
                .rfind('/')
                .with_context(|| "Path did not contain a folder")?
                + 1;
            let pre = &matched[*STORE_LEN..separator];
            let file = &matched[separator..matched.len() - 4];
            let formatted_path = format!(
                "{blue}{bold}{}{nobold}{}{reset}",
                pre,
                file,
                blue = color::Fg(color::Blue),
                bold = style::Bold,
                nobold = style::NoBold,
                reset = style::Reset,
            );

            println!("{}", formatted_path);
        }
    } else {
        return Err(PassrsError::NoMatchesFound(name).into());
    }

    Ok(())
}
