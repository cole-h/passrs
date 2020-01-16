// https://github.com/softprops/treeline/blob/eaaa03a5fac200fb5255c8aa927de43e7974745f/src/lib.rs
// Original work Copyright (c) 2015-2016 Doug Tangren
// Modified work Copyright (c) 2019 Cole Helbling
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::fmt;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use termion::color;
use termion::style;

use crate::consts::PASSWORD_STORE_DIR;

const EDGE: &str = "├── ";
const LINE: &str = "│   ";
const CORNER: &str = "└── ";
const BLANK: &str = "    ";

pub fn tree<P>(path: P) -> Result<Tree>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().canonicalize()?;

    let result = fs::read_dir(&path)?.filter_map(|e| e.ok()).fold(
        Tree {
            root: path,
            tree: Vec::new(),
        },
        |mut root, entry| {
            let meta = entry
                .metadata()
                .expect("Path doesn't exist (failed to get metadata)");

            if entry
                .file_name()
                .to_str()
                .map(|s| !s.starts_with('.'))
                .unwrap_or(false)
            {
                if meta.is_dir() {
                    root.tree
                        .push(tree(entry.path()).expect("Couldn't create branch"));
                } else {
                    root.tree.push(Tree {
                        root: entry.path(),
                        tree: Vec::new(),
                    });
                }
            }

            root
        },
    );

    Ok(result)
}

#[derive(Debug, Clone, Default)]
pub struct Tree {
    pub root: PathBuf,
    pub tree: Vec<Tree>,
}

impl Tree {
    fn draw_tree(f: &mut fmt::Formatter, leaves: &[Tree], prefix: Vec<bool>) -> fmt::Result {
        for (i, leaf) in leaves.iter().enumerate() {
            let last = i >= leaves.len() - 1;
            let mut prefix = prefix.clone();
            let leaf_name = leaf
                .root
                .file_name()
                .expect("Leaf didn't have a filename")
                .to_str()
                .expect("Couldn't convert filename to str");

            for s in &prefix {
                if *s {
                    write!(f, "{}", BLANK)?;
                } else {
                    write!(f, "{}", LINE)?;
                }
            }

            if last {
                if leaf.root.is_dir() {
                    writeln!(
                        f,
                        "{}{blue}{bold}{}{reset}",
                        CORNER,
                        leaf_name,
                        bold = style::Bold,
                        blue = color::Fg(color::Blue),
                        reset = style::Reset
                    )?;
                } else if leaf_name.ends_with(".gpg") {
                    writeln!(f, "{}{}", CORNER, &leaf_name[..leaf_name.len() - 4])?;
                } else {
                    writeln!(f, "{}{}", CORNER, leaf_name)?;
                }
            } else if leaf.root.is_dir() {
                writeln!(
                    f,
                    "{}{blue}{bold}{}{reset}",
                    EDGE,
                    leaf_name,
                    bold = style::Bold,
                    blue = color::Fg(color::Blue),
                    reset = style::Reset
                )?;
            } else if leaf_name.ends_with(".gpg") {
                writeln!(f, "{}{}", EDGE, &leaf_name[..leaf_name.len() - 4])?;
            } else {
                writeln!(f, "{}{}", EDGE, leaf_name)?;
            }

            if !leaf.tree.is_empty() {
                prefix.push(last);
                let _ = Self::draw_tree(f, &leaf.tree, prefix);
            }
        }
        write!(f, "")
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = if self.root == *PASSWORD_STORE_DIR {
            "Password Store"
        } else {
            self.root
                .file_name()
                .expect("Path didn't have a filename")
                .to_str()
                .expect("Couldn't convert filename to str")
        };

        if self.root.is_dir() {
            writeln!(
                f,
                "{bold}{blue}{}{reset}",
                name,
                bold = style::Bold,
                blue = color::Fg(color::Blue),
                reset = style::Reset
            )?;
        } else {
            writeln!(
                f,
                "{}",
                self.root
                    .file_name()
                    .expect("Leaf didn't have a filename")
                    .to_str()
                    .expect("Couldn't convert filename to str")
            )?;
        }

        Self::draw_tree(f, &self.tree, Vec::new())
    }
}
