extern crate clap;
extern crate git2;

use git2::{Repository, Remote, Error};
use clap::{Arg, App};
use std::path::Path;

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
    let mut remote = add_remote(&repo, submodule_dir).expect("Couldn't add a remote");
    // 2. Fetch submodule's history
    remote.fetch(&[], None, None).expect("Couldn't fetch submodule's history");
    // 3. Find out submodule's HEAD commit id
    let submodules = repo.submodules().expect("Couldn't obtain a list of submodules");
    let submodule_path = Path::new(submodule_dir);
    let submodule = submodules.iter().find(|s| s.path() == submodule_path)
        .expect("Couldn't find the submodule with expected path");
    let submodule_head = submodule.head_id()
        .expect("Couldn't obtain submodule's HEAD");
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

fn add_remote<'a>(repo : &'a Repository, submodule_name : &str) -> Result<Remote<'a>, Error> {
    // TODO: randomize remote's name or at least check that it doesn't exist already
    let url = String::from("./") + submodule_name;
    repo.remote(submodule_name, &url)
}
