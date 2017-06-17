extern crate clap;
extern crate git2;

use git2::{Repository, Remote, Error, Index, Commit};
use clap::{Arg, App};
use std::path::Path;
use std::collections::HashMap;

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
    let mut revwalk = repo.revwalk().expect("Couldn't obtain RevWalk object for the repo");
    revwalk.set_sorting(git2::SORT_REVERSE | git2::SORT_TOPOLOGICAL);
    revwalk.push(submodule_head).expect("Couldn't add submodule's HEAD to RevWalk list");

    let mut old_id_to_new = HashMap::new();

    for maybe_oid in revwalk {
        match maybe_oid {
            Ok(oid) => {
                // 4.1. Extract the tree
                let commit = repo.find_commit(oid).expect(&format!("Couldn't get a commit with ID {}", oid));
                let tree = commit.tree().expect(&format!("Couldn't obtain the tree of a commit with ID {}", oid));
                let mut old_index = Index::new().expect("Couldn't create an in-memory index for commit");
                let mut new_index = Index::new().expect("Couldn't create an in-memory index");
                old_index.read_tree(&tree).expect(&format!("Couldn't read the commit {} into index", oid));
                // 4.2. Obtain the new tree, where everything from the old one is moved under
                //   a directory named after the submodule
                for entry in old_index.iter() {
                    let mut new_entry = entry;

                    let mut new_path = String::from(submodule_dir);
                    new_path += "/";
                    new_path += &String::from_utf8(new_entry.path).expect("Failed to convert a path to str");

                    new_entry.path = new_path.into_bytes();
                    new_index.add(&new_entry).expect("Couldn't add an entry to the index");
                }
                let tree_id = new_index.write_tree_to(&repo).expect("Couldn't write the index into a tree");
                let tree = repo.find_tree(tree_id).expect("Couldn't retrieve the tree we just created");
                // 4.3. TODO: Create new commit with the new tree
                let parents = {
                    let mut p: Vec<Commit> = Vec::new();
                    for parent_id in commit.parent_ids() {
                        let new_parent_id = old_id_to_new[&parent_id];
                        let parent = repo.find_commit(new_parent_id).expect("Couldn't find parent commit by its id");
                        p.push(parent);
                    };
                    p
                };

                let mut parents_refs: Vec<&Commit> = Vec::new();
                for i in 0 .. parents.len() {
                    parents_refs.push(&parents[i]);
                }
                let new_commit_id = repo.commit(
                    None,
                    &commit.author(),
                    &commit.committer(),
                    &commit.message().expect("Couldn't retrieve commit's message"),
                    &tree,
                    &parents_refs[..])
                    .expect("Failed to commit");
                // 4.4. TODO: Update the map with the new commit's ID
                old_id_to_new.insert(oid, new_commit_id);
            },
            Err(e) =>
                eprintln!("Error walking the submodule's history: {:?}", e),
        }
    };
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
