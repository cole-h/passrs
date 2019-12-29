use std::fmt::{self, Display};
use std::fs;
use std::path::PathBuf;

use failure::{err_msg, Fallible};

const EDGE: &str = "├── ";
const LINE: &str = "│   ";
const CORNER: &str = "└── ";
const BLANK: &str = "    ";

// TODO: own tree using DFS
// maybe `petgraph` can help, at least to visualize
// TODO: https://docs.rs/ptree/0.2.1/ptree/ might be interesting

pub fn tree<P: Into<PathBuf>>(path: P) -> Fallible<Tree> {
    let path = path.into().canonicalize()?;
    let result = fs::read_dir(&path)?.filter_map(|e| e.ok()).fold(
        Tree(path, Vec::new()),
        |mut root, entry| {
            let dir = entry.metadata().unwrap();
            if entry
                .file_name()
                .to_str()
                .map(|s| !s.starts_with('.'))
                .unwrap_or(false)
            {
                if dir.is_dir() {
                    root.1.push(tree(entry.path()).unwrap());
                } else {
                    root.1.push(Tree(entry.path(), Vec::new()));
                }
            }
            root
        },
    );
    Ok(result)
}

// TODO: only show matching elements
pub fn find(search: &str, v: Tree) -> Fallible<Vec<Tree>> {
    let mut list = Vec::new();
    list.push(v.clone());

    for element in v.1 {
        let elt = element.0.to_str().unwrap_or("");
        let sep = elt.rfind('/').unwrap_or(0);
        let elt = &elt[sep..];
        if elt.contains(search) {
            list.push(element);
        // find(search, element)?;
        } else {
            // find(search, element)?;
            continue;
        }
    }

    Ok(list)
}

#[derive(Debug, Clone)]
pub struct Tree(PathBuf, pub Vec<Tree>);

impl Tree {
    fn draw_tree(f: &mut fmt::Formatter, leaves: &[Tree], prefix: Vec<bool>) -> fmt::Result {
        for (i, leaf) in leaves.iter().enumerate() {
            let last = i >= leaves.len() - 1;
            let mut prefix = prefix.clone();
            let leaf_name = leaf.0.file_name().unwrap().to_str().unwrap();
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
                } else {
                    if leaf_name.ends_with(".gpg") {
                        writeln!(f, "{}{}", CORNER, &leaf_name[..leaf_name.len() - 4])?;
                    } else {
                        writeln!(f, "{}{}", CORNER, leaf_name)?;
                    }
                }
            } else {
                if leaf.0.is_dir() {
                    writeln!(
                        f,
                        "{}{blue}{bold}{}{reset}",
                        EDGE,
                        leaf_name,
                        bold = style::Bold,
                        blue = color::Fg(color::Blue),
                        reset = style::Reset
                    )?;
                } else {
                    if leaf_name.ends_with(".gpg") {
                        writeln!(f, "{}{}", EDGE, &leaf_name[..leaf_name.len() - 4])?;
                    } else {
                        writeln!(f, "{}{}", EDGE, leaf_name)?;
                    }
                }
            }

            if !leaf.1.is_empty() {
                prefix.push(last);
                let _ = Self::draw_tree(f, &leaf.1, prefix);
            }
        }
        write!(f, "")
    }
}

use termion::color;
use termion::style;

impl Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_dir() {
            writeln!(
                f,
                "{bold}{blue}{}{reset}",
                self.0.file_name().unwrap().to_str().unwrap(),
                bold = style::Bold,
                blue = color::Fg(color::Blue),
                reset = style::Reset
            )?;
        } else {
            writeln!(f, "{}", self.0.file_name().unwrap().to_str().unwrap())?;
        }

        Self::draw_tree(f, &self.1, Vec::new())
    }
}

// TODO: make this only tree the specified dir. no recursion
/// https://github.com/kddeisz/tree
pub fn _draw_tree(dir: &str, prefix: &str) -> Fallible<()> {
    let dir = if fs::metadata(dir)?.is_file() {
        let sep = dir.rfind('/').unwrap_or(0);
        &dir[..sep]
    } else {
        dir
    };

    let mut paths: Vec<_> = fs::read_dir(dir)?
        .map(|entry| entry.unwrap().path())
        .collect();
    let mut index = paths.len();

    paths.sort_by(|a, b| a.cmp(b));

    for path in paths {
        let name = path
            .file_name()
            .ok_or_else(|| err_msg("Path did not contain a value"))?
            .to_str()
            .ok_or_else(|| err_msg("Path did not contain a value"))?;
        index -= 1;

        if name.starts_with('.') {
            continue;
        }

        let name = if name.ends_with(".gpg") {
            &name[..name.len() - 4]
        } else {
            name
        };

        if index == 0 {
            println!("{}{}{}", prefix, CORNER, name);
        // if path.is_dir() {
        //     // TODO: blue and bold dir name
        //     draw_tree(
        //         &format!("{}/{}", dir, name),
        //         &format!("{}{}", prefix, BLANK),
        //     )?;
        // }
        } else {
            println!("{}{}{}", prefix, EDGE, name);
            // if path.is_dir() {
            //     // TODO: blue and bold dir name
            //     draw_tree(&format!("{}/{}", dir, name), &format!("{}{}", prefix, LINE))?;
            // }
        }
    }

    Ok(())
}
