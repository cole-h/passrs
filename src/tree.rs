//! Happy little accidents
//!
//! # tree

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

use std::fs;
use std::io;
use std::io::Write;
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

    let tree = fs::read_dir(&path)?.filter_map(|e| e.ok()).fold(
        Tree {
            root: path,
            leaves: Vec::new(),
        },
        |mut root, entry| {
            let meta = entry
                .metadata()
                .expect("Path doesn't exist (failed to get metadata)");
            let show = entry
                .file_name()
                .to_str()
                .map(|s| !s.starts_with('.'))
                .unwrap_or(false);

            if show {
                if meta.is_dir() {
                    root.leaves
                        .push(tree(entry.path()).expect("Couldn't create leaves"));
                } else {
                    root.leaves.push(Tree {
                        root: entry.path(),
                        leaves: Vec::new(),
                    });
                }
            }

            root
        },
    );

    Ok(tree)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tree {
    pub root: PathBuf,
    pub leaves: Vec<Tree>,
}

impl Tree {
    fn root_path(&self) -> String {
        self.root.display().to_string()
    }

    fn draw_tree(mut branches: Vec<Tree>, prefix: Vec<bool>) -> Result<()> {
        branches.sort_by(|a, b| {
            a.root_path()
                .to_ascii_lowercase()
                .cmp(&b.root_path().to_ascii_lowercase())
        });

        for (i, branch) in branches.iter().enumerate() {
            let last = i >= branches.len() - 1;
            let mut prefix = prefix.clone();
            let leaf_name = branch
                .root
                .file_name()
                .expect("Leaf didn't have a filename")
                .to_str()
                .expect("Couldn't convert filename to str");

            for pre in &prefix {
                if *pre {
                    write!(io::stdout(), "{}", BLANK)?;
                } else {
                    write!(io::stdout(), "{}", LINE)?;
                }
            }

            if last {
                if branch.root.is_dir() {
                    writeln!(
                        io::stdout(),
                        "{}{blue}{bold}{}{reset}",
                        CORNER,
                        leaf_name,
                        bold = style::Bold,
                        blue = color::Fg(color::Blue),
                        reset = style::Reset
                    )?;
                } else {
                    // if the leaf ends with .gpg, don't show that
                    let leaf_name =
                        &leaf_name[..leaf_name.rfind(".gpg").unwrap_or_else(|| leaf_name.len())];

                    writeln!(io::stdout(), "{}{}", CORNER, leaf_name)?;
                }
            } else if branch.root.is_dir() {
                writeln!(
                    io::stdout(),
                    "{}{blue}{bold}{}{reset}",
                    EDGE,
                    leaf_name,
                    bold = style::Bold,
                    blue = color::Fg(color::Blue),
                    reset = style::Reset
                )?;
            } else {
                // if the leaf ends with .gpg, don't show that
                let leaf_name =
                    &leaf_name[..leaf_name.rfind(".gpg").unwrap_or_else(|| leaf_name.len())];

                writeln!(io::stdout(), "{}{}", EDGE, leaf_name)?;
            }

            if !branch.leaves.is_empty() {
                prefix.push(last);
                Tree::draw_tree(branch.leaves.clone(), prefix)?;
            }
        }

        Ok(())
    }

    pub fn display_tree(&self) -> Result<()> {
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
                io::stdout(),
                "{bold}{blue}{}{reset}",
                name,
                bold = style::Bold,
                blue = color::Fg(color::Blue),
                reset = style::Reset
            )?;
        } else {
            writeln!(
                io::stdout(),
                "{}",
                self.root
                    .file_name()
                    .expect("Leaf didn't have a filename")
                    .to_str()
                    .expect("Couldn't convert filename to str")
            )?;
        }

        Tree::draw_tree(self.leaves.clone(), Vec::new())?;

        Ok(())
    }
}
