extern crate clap;
extern crate git2;

use git2::{Repository, Remote, Error, Index, Commit};
use git2::build::CheckoutBuilder;
use clap::{Arg, App};
use std::path::Path;
use std::collections::{HashMap, HashSet};

const E_NO_GIT_REPO: i32 = 1;

fn main() {
    let exit_code = real_main();
    std::process::exit(exit_code);
}

fn real_main() -> i32 {
    let options = App::new("git-submerge")
                          .version("0.1")
                          .author("Alexander Batischev <eual.jp@gmail.com>")
                          // TODO: get this in synch with Cargo.toml and README
                          .about("Merges git submodule into the repo as if it was that way \
                                  from the start")
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
            eprintln!("Couldn't find Git repo in the current directory: {}",
                      e.message());
            return E_NO_GIT_REPO;
        }
    };

    // 1. Add submodule as a remote
    let mut remote = add_remote(&repo, submodule_dir).expect("Couldn't add a remote");
    // 2. Fetch submodule's history
    remote.fetch(&[], None, None).expect("Couldn't fetch submodule's history");
    // 3. Find out submodule's HEAD commit id
    let submodule_path = Path::new(submodule_dir);
    let submodule = repo.find_submodule(&submodule_dir)
        .expect("Couldn't find the submodule with expected path");
    let submodule_head = submodule.head_id()
        .expect("Couldn't obtain submodule's HEAD");
    // 4. Rewrite submodule branch's history, moving everything under a single directory named
    //    after the submodule
    let mut submodule_revwalk = get_submodule_revwalk(&repo, &submodule_head);

    let mut old_id_to_new = HashMap::new();

    rewrite_submodule_history(&repo,
                              &mut submodule_revwalk,
                              &mut old_id_to_new,
                              &submodule_dir);
    // 7. Remove submodule's remote
    repo.remote_delete(submodule_dir).expect("Couldn't remove submodule's remote");
    // 8. Run through master's history, doing two things:
    let repo_revwalk = get_repo_revwalk(&repo);

    for maybe_oid in repo_revwalk {
        match maybe_oid {
            Ok(oid) => {
                // 8.1 updating the tree to contain the relevant tree from submodule
                let commit = repo.find_commit(oid)
                    .expect(&format!("Couldn't get a commit with ID {}", oid));
                let tree = commit.tree()
                    .expect(&format!("Couldn't obtain the tree of a commit with ID {}", oid));

                let submodule_subdir = match tree.get_path(submodule_path) {
                    Ok(tree) => tree,
                    Err(e) => {
                        if e.code() == git2::ErrorCode::NotFound &&
                           e.class() == git2::ErrorClass::Tree {
                            // It's okay. The tree lacks the subtree corresponding to the
                            // submodule. In other words, the commit doesn't include the submodule.
                            // That's totally fine. Let's map it into itself and move on.
                            old_id_to_new.insert(oid, oid);
                            continue;
                        } else {
                            // Unexpected error; let's report it and abort the program
                            // TODO: clean things up before aborting
                            panic!("Error getting submodule's subdir from the tree: {:?}", e);
                        };
                    }
                };

                // **INVARIANT**: if we got this far, current commit contains a submodule and
                // should be rewritten

                let submodule_commit_id = submodule_subdir.id();
                let new_submodule_commit_id = old_id_to_new[&submodule_commit_id];
                let submodule_commit = repo.find_commit(new_submodule_commit_id)
                    .expect("Couldn't obtain submodule's commit");
                let subtree_id = submodule_commit.tree()
                    .and_then(|t| t.get_path(submodule_path))
                    .and_then(|te| Ok(te.id()))
                    .expect("Couldn't obtain submodule's subtree ID");

                let mut treebuilder = repo.treebuilder(Some(&tree))
                    .expect("Couldn't create TreeBuilder");
                treebuilder.remove(submodule_path)
                    .expect("Couldn't remove submodule path from TreeBuilder");
                treebuilder.insert(submodule_path, subtree_id, 0o040000)
                    .expect("Couldn't add submodule as a subdir to TreeBuilder");
                let new_tree_id = treebuilder.write()
                    .expect("Couldn't write TreeBuilder into a Tree");
                let new_tree = repo.find_tree(new_tree_id)
                    .expect("Couldn't read back the Tree we just wrote");

                // 8.2 in commits that used to update the submodule, add a parent pointing to
                //   appropriate commit in new submodule history
                let mut parent_subtree_ids = HashSet::new();
                for parent in commit.parents() {
                    let parent_tree = parent.tree().expect("Couldn't obtain parent's tree");
                    let parent_subdir_tree_id = parent_tree.get_path(submodule_path)
                        .and_then(|x| Ok(x.id()));

                    match parent_subdir_tree_id {
                        Ok(id) => {
                            parent_subtree_ids.insert(id);
                            ()
                        }
                        Err(e) => {
                            if e.code() == git2::ErrorCode::NotFound &&
                               e.class() == git2::ErrorClass::Tree {
                                // It's okay; carry on.
                                continue;
                            } else {
                                panic!("Error getting submodule's subdir from the tree: {:?}", e);
                            };
                        }
                    }
                }
                // true if
                //
                // o--o--o--A--
                //             `,-C
                //  o--o--o--B-
                //
                //  or
                //
                // o--o--o--o--A--B
                //
                // false if
                //
                // o--o--o--A--
                //             `,-A
                //  o--o--o--B-
                //
                //  or
                //
                // o--o--o--A--
                //             `,-B
                //  o--o--o--B-
                let submodule_updated: bool = !parent_subtree_ids.contains(&submodule_commit_id);

                // rewrite the parents if the submodule was updated
                let parents = {
                    let mut p: Vec<Commit> = Vec::new();
                    for parent_id in commit.parent_ids() {
                        let actual_parent_id = old_id_to_new[&parent_id];
                        let parent = repo.find_commit(actual_parent_id)
                            .expect("Couldn't find parent commit by its id");
                        p.push(parent);
                    }

                    if submodule_updated {
                        p.push(submodule_commit);
                    }

                    p
                };

                let mut parents_refs: Vec<&Commit> = Vec::new();
                for i in 0..parents.len() {
                    parents_refs.push(&parents[i]);
                }
                let new_commit_id = repo.commit(None,
                            &commit.author(),
                            &commit.committer(),
                            &commit.message().expect("Couldn't retrieve commit's message"),
                            &new_tree,
                            &parents_refs[..])
                    .expect("Failed to commit");

                old_id_to_new.insert(oid, new_commit_id);
            }
            Err(e) => eprintln!("Error walking the submodule's history: {:?}", e),
        }
    }

    // It's safe to do force-reset because we checked at the beginning and the repo was clean
    let mut checkoutbuilder = CheckoutBuilder::new();
    checkoutbuilder.force();

    let head = repo.head().expect("Couldn't obtain repo's HEAD");
    let head_id = head.target().expect("Couldn't resolve repo's HEAD to a commit ID");
    let updated_id = old_id_to_new[&head_id];
    let object = repo.find_object(updated_id, None)
        .expect("Couldn't look up an object at which HEAD points");
    repo.reset(&object, git2::ResetType::Hard, Some(&mut checkoutbuilder))
        .expect("Couldn't run force-reset");

    0 // An exit code indicating success
}

fn add_remote<'a>(repo: &'a Repository, submodule_name: &str) -> Result<Remote<'a>, Error> {
    // TODO: randomize remote's name or at least check that it doesn't exist already
    // Maybe use remote_anonymous()
    let url = String::from("./") + submodule_name;
    repo.remote(submodule_name, &url)
}

fn get_submodule_revwalk<'repo>(repo: &'repo git2::Repository,
                                submodule_head: &git2::Oid)
                                -> git2::Revwalk<'repo> {
    let mut revwalk = repo.revwalk().expect("Couldn't obtain RevWalk object for the repo");
    // "Topological" and reverse means "parents are always visited before their children".
    // We need that in order to be sure that our old-to-new-ids map always contains everything we
    // need it to contain.
    revwalk.set_sorting(git2::SORT_REVERSE | git2::SORT_TOPOLOGICAL);
    // TODO: push all branches and tags, not just HEAD
    revwalk.push(*submodule_head).expect("Couldn't add submodule's HEAD to RevWalk list");

    revwalk
}

fn rewrite_submodule_history(repo: &git2::Repository,
                             revwalk: &mut git2::Revwalk,
                             old_id_to_new: &mut HashMap<git2::Oid, git2::Oid>,
                             submodule_dir: &str) {
    for maybe_oid in revwalk {
        match maybe_oid {
            Ok(oid) => {
                // 4.1. Extract the tree
                let commit = repo.find_commit(oid)
                    .expect(&format!("Couldn't get a commit with ID {}", oid));
                let tree = commit.tree()
                    .expect(&format!("Couldn't obtain the tree of a commit with ID {}", oid));
                let mut old_index = Index::new()
                    .expect("Couldn't create an in-memory index for commit");
                let mut new_index = Index::new().expect("Couldn't create an in-memory index");
                old_index.read_tree(&tree)
                    .expect(&format!("Couldn't read the commit {} into index", oid));
                // 4.2. Obtain the new tree, where everything from the old one is moved under
                //   a directory named after the submodule
                for entry in old_index.iter() {
                    let mut new_entry = entry;

                    // TODO: what mode, owner, mtime etc. does the newly created dir get?
                    let mut new_path = String::from(submodule_dir);
                    new_path += "/";
                    new_path += &String::from_utf8(new_entry.path)
                        .expect("Failed to convert a path to str");

                    new_entry.path = new_path.into_bytes();
                    new_index.add(&new_entry).expect("Couldn't add an entry to the index");
                }
                let tree_id = new_index.write_tree_to(&repo)
                    .expect("Couldn't write the index into a tree");
                old_id_to_new.insert(tree.id(), tree_id);
                let tree = repo.find_tree(tree_id)
                    .expect("Couldn't retrieve the tree we just created");
                // 4.3. Create new commit with the new tree
                let parents = {
                    let mut p: Vec<Commit> = Vec::new();
                    for parent_id in commit.parent_ids() {
                        let new_parent_id = old_id_to_new[&parent_id];
                        let parent = repo.find_commit(new_parent_id)
                            .expect("Couldn't find parent commit by its id");
                        p.push(parent);
                    }
                    p
                };

                let mut parents_refs: Vec<&Commit> = Vec::new();
                for i in 0..parents.len() {
                    parents_refs.push(&parents[i]);
                }
                let new_commit_id = repo.commit(None,
                            &commit.author(),
                            &commit.committer(),
                            &commit.message().expect("Couldn't retrieve commit's message"),
                            &tree,
                            &parents_refs[..])
                    .expect("Failed to commit");
                // 4.4. Update the map with the new commit's ID
                old_id_to_new.insert(oid, new_commit_id);
            }
            Err(e) => eprintln!("Error walking the submodule's history: {:?}", e),
        }
    }
}

fn get_repo_revwalk<'repo>(repo: &'repo git2::Repository) -> git2::Revwalk<'repo> {
    let mut revwalk = repo.revwalk().expect("Couldn't obtain RevWalk object for the repo");
    revwalk.set_sorting(git2::SORT_REVERSE | git2::SORT_TOPOLOGICAL);
    let head = repo.head().expect("Couldn't obtain repo's HEAD");
    let head_id = head.target().expect("Couldn't resolve repo's HEAD to a commit ID");
    // TODO: push all branches and tags, not just HEAD
    revwalk.push(head_id).expect("Couldn't add repo's HEAD to RevWalk list");

    revwalk
}
