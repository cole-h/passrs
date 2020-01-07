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
        Tree(path, Vec::new()),
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
                    root.1
                        .push(tree(entry.path()).expect("Couldn't create branch"));
                } else {
                    root.1.push(Tree(entry.path(), Vec::new()));
                }
            }
            root
        },
    );
    Ok(result)
}

#[derive(Debug, Clone, Default)]
pub struct Tree(pub PathBuf, pub Vec<Tree>);

impl Tree {
    fn draw_tree(f: &mut fmt::Formatter, leaves: &[Tree], prefix: Vec<bool>) -> fmt::Result {
        for (i, leaf) in leaves.iter().enumerate() {
            let last = i >= leaves.len() - 1;
            let mut prefix = prefix.clone();
            let leaf_name = leaf
                .0
                .file_name()
                .expect("Leaf didn't have a filename")
                .to_str()
                .expect("Couldn't convert filename to str");

            // If the user has enabled signing, every .sig file will appear in
            // the tree. We don't want that, so just continue if we hit one.
            if leaf_name.ends_with(".sig") {
                continue;
            }
            for s in &prefix {
                if *s {
                    write!(f, "{}", BLANK)?;
                } else {
                    write!(f, "{}", LINE)?;
                }
            }
            if last {
                if leaf.0.is_dir() {
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
            } else if leaf.0.is_dir() {
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

            if !leaf.1.is_empty() {
                prefix.push(last);
                let _ = Self::draw_tree(f, &leaf.1, prefix);
            }
        }
        write!(f, "")
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = if self.0 == PathBuf::from(&*PASSWORD_STORE_DIR) {
            "Password Store"
        } else {
            self.0
                .file_name()
                .expect("Path didn't have a filename")
                .to_str()
                .expect("Couldn't convert filename to str")
        };

        if self.0.is_dir() {
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
                self.0
                    .file_name()
                    .expect("Leaf didn't have a filename")
                    .to_str()
                    .expect("Couldn't convert filename to str")
            )?;
        }

        Self::draw_tree(f, &self.1, Vec::new())
    }
}
