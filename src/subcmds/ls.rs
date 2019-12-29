use failure::Fallible;
use termion::color;
use termion::style;

use crate::consts::PASSWORD_STORE_DIR;
use crate::tree;

pub fn ls(path: Option<String>) -> Fallible<()> {
    let root = if let Some(path) = path {
        // TODO: blue and bold
        println!(
            "{blue}{bold}{}{reset}",
            path,
            blue = color::Fg(color::Blue),
            bold = style::Bold,
            reset = style::Reset
        );

        if path.contains(&*PASSWORD_STORE_DIR) {
            path
        } else {
            [PASSWORD_STORE_DIR.to_owned(), path].concat()
        }
    } else {
        // TODO: blue and bold
        println!(
            "{blue}{bold}Password Store{reset}",
            blue = color::Fg(color::Blue),
            bold = style::Bold,
            reset = style::Reset
        );

        PASSWORD_STORE_DIR.to_owned()
    };

    // no GUI select -- just tell us if entry can't be found
    // tree::draw_tree(&root, "")?;

    Ok(())
}
