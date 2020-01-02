use failure::Fallible;

use crate::consts::PASSWORD_STORE_DIR;
use crate::util;
use crate::PassrsError;

pub fn find(name: String) -> Fallible<()> {
    let matches = util::find_target_single(&name)?;

    // TODO: don't show folders or files that don't end with gpg
    // let matches = matches
    //     .iter()
    //     .filter(|e| e.ends_with(".gpg"))
    //     .map(ToOwned::to_owned)
    //     .collect::<Vec<_>>();

    // FIXME: For now, just following `gopass`s lead: straightup print the matches to stdout.
    // Printing trees is a pain in the ass. Filtering them is even worse.

    // TODO: color the entry. blue-bold for folder, blue-normal for file
    if matches.len() >= 1 {
        for matched in &matches {
            if matched.ends_with(".gpg") {
                println!("{}", &matched[PASSWORD_STORE_DIR.len()..matched.len() - 4]);
            } else {
                println!("{}", &matched[PASSWORD_STORE_DIR.len()..]);
            }
        }
    } else if matches.len() < 1 {
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
