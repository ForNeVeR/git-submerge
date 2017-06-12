extern crate clap;
extern crate git2;

use git2::Repository;
use clap::{Arg, App};

const E_NO_GIT_REPO : i32 = 1;

fn main() {
    let options = App::new("git-submerge")
                          .version("0.1")
                          .author("Alexander Batischev <eual.jp@gmail.com>")
                          .about("Merges git submodule into the repo as if it was that way from the start")
                          .arg(Arg::with_name("SUBMODULE_DIR")
                               .help("The submodule to merge")
                               .required(true)
                               .index(1))
                          .get_matches();
    // We can safely use unwrap() here because if the option is empty, Clap would've already shown
    // the error message and aborted.
    let submodule_dir = options.value_of("SUBMODULE_DIR").unwrap();
    println!("Merging {}...", submodule_dir);

    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Couldn't find Git repo in the current directory: {}", e.message());
            std::process::exit(E_NO_GIT_REPO);
        },
    };

    // 1. Add submodule as a remote
    // 2. Fetch submodule's history
    // 3. Check out submodule's branch under some unique name (UUID?)
    // 4. Rewrite submodule branch's history, moving everything under a single directory named
    //    after the submodule
    // 5. Run through main branch's history and note down commit IDs where submodule was touched,
    //    along with submodule's commit ID
    // 6. Run through old and new submodule's history (in lockstep) and note down new commit IDs of
    //    the commits that were referenced in the main repo
    // 7. Remove submodule's remote
    // 8. Run through master's history, doing two things:
    //      8.1 updating the tree to contain the relevant tree from submodule
    //      8.2 in commits that used to update the submodule, add a parent pointing to appropriate
    //          commit in new submodule history
}
