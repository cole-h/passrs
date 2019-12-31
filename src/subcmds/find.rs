use failure::Fallible;

use crate::consts::PASSWORD_STORE_DIR;
// use crate::ui::{self, UiResult};
use crate::util;

pub fn find(names: Vec<String>) -> Fallible<()> {
    // TODO: make a tree function that trees to a specific file/folder and  use that here
    let matches = util::find_target_multi(names)?;

    // FIXME: For now, just following `gopass`s lead: straightup print the matches to stdout.
    // Printing trees is a pain in the ass. Filtering them is even worse.
    // ref: https://github.com/gopasspw/gopass/blob/82c67a8a465d4e7d1cbf497c7fba26e6429944e0/pkg/action/find.go
    // TODO: fuzzy search feature -- get 5 closest matches
    for m in matches {
        if m.ends_with(".gpg") {
            println!("{}", &m[PASSWORD_STORE_DIR.len()..m.len() - 4]);
        }
        // } else {
        //     m.len()
        // };
    }

    // let file = ui::display_matches_for_targets(matches)?;
    // match file {
    //     UiResult::Success(file) => {
    //         let password = util::decrypt_file_into_vec(file)?;
    //         for line in password {
    //             println!("{}", line);
    //         }
    //     }
    //     UiResult::CopiedToClipboard(file) => {
    //         println!("{}", &file[..file.len() - 4]);
    //     }
    //     _ => {}
    // }

    Ok(())
}
