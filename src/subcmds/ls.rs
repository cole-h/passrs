use anyhow::Result;

use crate::consts::STORE_STRING;
use crate::tree;

pub fn ls(path: Option<String>) -> Result<()> {
    let root = if let Some(path) = path {
        if path.contains(&*STORE_STRING) {
            path
        } else {
            [&*STORE_STRING, path.as_str()].concat()
        }
    } else {
        STORE_STRING.to_owned()
    };

    let tree = tree::tree(&root)?;
    if tree.tree.is_empty() {
        return Ok(());
    }

    println!("{}", tree);

    Ok(())
}
