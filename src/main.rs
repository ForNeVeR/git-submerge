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
}
