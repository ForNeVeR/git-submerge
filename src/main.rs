extern crate clap;
extern crate git2;

use git2::Repository;
use clap::{Arg, App};

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
    match options.value_of("SUBMODULE_DIR") {
        Some(submodule_dir) => println!("Merging {}...", submodule_dir),
        None => panic!("no submodule name specified!"),
    };

    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open the repository in the current directory: {}", e),
    };
}
