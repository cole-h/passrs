use std::path::PathBuf;

use anyhow::Result;

use crate::consts::STORE_STRING;
use crate::tree;
use crate::PassrsError;

pub(crate) fn ls(path: Option<String>) -> Result<()> {
    let root = if let Some(path) = path {
        if path.contains(&*STORE_STRING) {
            path
        } else {
            [&*STORE_STRING, "/", path.as_str()].concat()
        }
    } else {
        STORE_STRING.to_owned()
    };

    if PathBuf::from(&root).exists() {
        let tree = tree::tree(&root)?;

        if tree.leaves.is_empty() {
            return Ok(());
        } else {
            println!("{}", tree);
        }

        Ok(())
    } else {
        Err(PassrsError::PathDoesntExist(root).into())
    }
}
