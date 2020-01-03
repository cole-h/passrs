use std::fmt::{self, Display};
use std::fs;
use std::path::PathBuf;

use failure::Fallible;
use termion::{color, style};

const EDGE: &str = "├── ";
const LINE: &str = "│   ";
const CORNER: &str = "└── ";
const BLANK: &str = "    ";

// TODO: make the tree from the path's components -- HashMap? anything will need indirection
// ignore::WalkBuilder might help with the "only include matches" scenario

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

#[derive(Debug, Clone, Default)]
pub struct Tree(pub PathBuf, pub Vec<Tree>);

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

// https://github.com/kddeisz/tree

// TODO: petgraph
// use petgraph::graph::NodeIndex;
// use petgraph::stable_graph::StableGraph;
// use petgraph::visit::Dfs;
// use std::collections::HashMap;

// pub struct Graph {
//     pub graph: StableGraph<std::path::PathBuf, usize>, // usize = depth of pathbuf
//     pub nodes: HashMap<std::path::PathBuf, NodeIndex>, // usize = index of child (child number)
//     pub root: Option<std::path::PathBuf>,
// }
