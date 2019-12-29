use std::fs;
use std::path::Path;

use failure::Fallible;
use walkdir::{DirEntry, WalkDir};

use crate::tree::Tree;

pub fn find(names: Vec<String>) -> Fallible<()> {
    // TODO: `tree` to any matching files (folder name, file name, etc)
    // currently, tree only works for exact paths

    // use walkdir to find any file/dir that matches, and push it onto a vec of (Path, IsDir(bool))
    // then iter over that vec
    //
    // or walkdir and filter(_map) it with .contains(names), then check if it's
    // a dir, tree it

    let dir = "/tmp/passrstest";
    // for (idx, entry) in WalkDir::new(dir)
    //     .into_iter()
    //     .filter_entry(|e| !is_hidden(e))
    //     .enumerate()
    // {
    //     let entry = entry?;
    //     dbg!((idx, &entry));
    //     let path = entry.path().to_str().unwrap();

    //     for name in &names {
    //         if path.contains(name) {
    //             //
    //         }
    //     }
    // }

    // tree::draw_tree(dir, "")?;
    // println!("{}", tree(dir)?);

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}
