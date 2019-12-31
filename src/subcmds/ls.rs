use failure::Fallible;

use crate::consts::PASSWORD_STORE_DIR;
use crate::tree;

pub fn ls(path: Option<String>) -> Fallible<()> {
    let root = if let Some(path) = path {
        if path.contains(&*PASSWORD_STORE_DIR) {
            path
        } else {
            [PASSWORD_STORE_DIR.to_owned(), path].concat()
        }
    } else {
        PASSWORD_STORE_DIR.to_owned()
    };

    // no GUI select -- just tell us if entry can't be found
    let tree = tree::tree(&root)?;
    if tree.1.len() == 0 {
        // TODO: we don't show single-element trees
        return Ok(());
    }

    println!("{}", tree);

    Ok(())
}
