use failure::Fallible;

use crate::consts::PASSWORD_STORE_DIR;
use crate::util;
use crate::PassrsError;

pub fn find(name: String) -> Fallible<()> {
    let matches = util::find_target_single(&name)?;

    // FIXME: For now, just following `gopass`s lead: straightup print the matches to stdout.
    // Printing trees is a pain in the ass. Filtering them is even worse.

    // 1. get a list of file/directories that match the given search string
    //   a. if dir, add to vec
    //   b. if file, add parent dir to vec
    // (goal is to have ONLY dirs in the vec)
    // 2. dirs.iter().filter_map(|e| e.as_ref().ok()).fold(Tree(PathBuf::from(&*PASSWORD_STORE_DIR), Vec::new()),
    //       |mut root, entry| { ... }

    // TODO: color the entry. blue-bold for folder, blue-normal for file
    if !matches.is_empty() {
        for matched in &matches {
            if matched.ends_with(".gpg") {
                println!("{}", &matched[PASSWORD_STORE_DIR.len()..matched.len() - 4]);
            } else {
                println!("{}", &matched[PASSWORD_STORE_DIR.len()..]);
            }
        }
    } else if matches.is_empty() {
        let fuzzy = fuzz_search::best_matches(&name, matches, 5).collect::<Vec<_>>();

        if fuzzy.is_empty() {
            return Err(PassrsError::NoMatchesFound(name).into());
        } else {
            for found in fuzzy {
                if found.ends_with(".gpg") {
                    println!("{}", &found[PASSWORD_STORE_DIR.len()..found.len() - 4]);
                } else {
                    println!("{}", &found[PASSWORD_STORE_DIR.len()..]);
                }
            }
        }
    } else {
        return Err(PassrsError::NoMatchesFound(name).into());
    }

    Ok(())
}
