use anyhow::Result;

use crate::consts::PASSWORD_STORE_STRING;
use crate::tree;

pub fn ls(path: Option<String>) -> Result<()> {
    let root = if let Some(path) = path {
        if path.contains(&*PASSWORD_STORE_STRING) {
            path
        } else {
            [&*PASSWORD_STORE_STRING, path.as_str()].concat()
        }
    } else {
        PASSWORD_STORE_STRING.to_owned()
    };

    // no GUI select -- just tell us if entry can't be found
    let tree = tree::tree(&root)?;
    if tree.1.is_empty() {
        // we don't show single-element trees
        return Ok(());
    }

    println!("{}", tree);

    Ok(())
}
