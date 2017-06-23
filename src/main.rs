extern crate clap;
extern crate git2;

use git2::{Repository, Commit, Oid, Revwalk, Index};
use std::collections::HashMap;

#[macro_use]
mod macros;

const E_SUCCESS: i32 = 0;
const E_NO_GIT_REPO: i32 = 1;

fn main() {
    let exit_code = real_main();
    std::process::exit(exit_code);
}

fn real_main() -> i32 {
    let submodule_dir = parse_cli_arguments();
    println!("Merging {}...", submodule_dir);

    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Couldn't find Git repo in the current directory: {}",
                      e.message());
            return E_NO_GIT_REPO;
        }
    };

    let mut submodule_revwalk = get_submodule_revwalk(&repo, &submodule_dir);

    let mut old_id_to_new = HashMap::new();

    rewrite_submodule_history(&repo,
                              &mut submodule_revwalk,
                              &mut old_id_to_new,
                              &submodule_dir);

    let mut repo_revwalk = get_repo_revwalk(&repo);
    rewrite_repo_history(&repo, &mut repo_revwalk, &mut old_id_to_new, &submodule_dir);

    checkout_rewritten_history(&repo, &old_id_to_new);

    E_SUCCESS
}

fn parse_cli_arguments() -> String {
    let options = clap::App::new("git-submerge")
                          .version("0.1")
                          .author("Alexander Batischev <eual.jp@gmail.com>")
                          .about("Merge Git submodule into the main repo \
                                  as if they've never been separate at all")
                          .arg(clap::Arg::with_name("SUBMODULE_DIR")
                               .help("The submodule to merge")
                               .required(true)
                               .index(1))
                          .get_matches();

    // We can safely use unwrap() here because the argument is marked as "required" and Clap checks
    // its presence for us.
    String::from( options.value_of("SUBMODULE_DIR").unwrap() )
}

fn get_submodule_revwalk<'repo>(repo: &'repo Repository, submodule_dir: &str) -> Revwalk<'repo> {
    let submodule_url = String::from("./") + submodule_dir;
    let mut remote = repo.remote_anonymous(&submodule_url).expect("Couldn't create an anonymous remote");
    remote.fetch(&[], None, None).expect("Couldn't fetch submodule's history");
    let submodule = repo.find_submodule(submodule_dir)
        .expect("Couldn't find the submodule with expected path");
    let submodule_head = submodule.head_id()
        .expect("Couldn't obtain submodule's HEAD");

    let mut revwalk = repo.revwalk().expect("Couldn't obtain RevWalk object for the repo");
    // "Topological" and reverse means "parents are always visited before their children".
    // We need that in order to be sure that our old-to-new-ids map always contains everything we
    // need it to contain.
    revwalk.set_sorting(git2::SORT_REVERSE | git2::SORT_TOPOLOGICAL);
    // TODO (#6): push all branches and tags, not just HEAD
    revwalk.push(submodule_head).expect("Couldn't add submodule's HEAD to RevWalk list");

    revwalk
}

fn rewrite_submodule_history(repo: &Repository,
                             revwalk: &mut Revwalk,
                             old_id_to_new: &mut HashMap<Oid, Oid>,
                             submodule_dir: &str) {
    for maybe_oid in revwalk {
        match maybe_oid {
            Ok(oid) => {
                let commit = repo.find_commit(oid)
                    .expect(&format!("Couldn't get a commit with ID {}", oid));
                let tree = commit.tree()
                    .expect(&format!("Couldn't obtain the tree of a commit with ID {}", oid));
                // TODO (#10): consider using TreeBuilder instead of in-memory index
                let mut old_index = Index::new()
                    .expect("Couldn't create an in-memory index for commit");
                let mut new_index = Index::new().expect("Couldn't create an in-memory index");
                old_index.read_tree(&tree)
                    .expect(&format!("Couldn't read the commit {} into index", oid));

                // Obtain the new tree, where everything from the old one is moved under
                // a directory named after the submodule
                for entry in old_index.iter() {
                    let mut new_entry = entry;

                    // TODO (#11): what mode, owner, mtime etc. does the newly created dir get?
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

                old_id_to_new.insert(oid, new_commit_id);
            }
            Err(e) => eprintln!("Error walking the submodule's history: {:?}", e),
        }
    }
}

fn get_repo_revwalk<'repo>(repo: &'repo Repository) -> Revwalk<'repo> {
    let mut revwalk = repo.revwalk().expect("Couldn't obtain RevWalk object for the repo");
    revwalk.set_sorting(git2::SORT_REVERSE | git2::SORT_TOPOLOGICAL);
    let head = repo.head().expect("Couldn't obtain repo's HEAD");
    let head_id = head.target().expect("Couldn't resolve repo's HEAD to a commit ID");
    // TODO (#6): push all branches and tags, not just HEAD
    revwalk.push(head_id).expect("Couldn't add repo's HEAD to RevWalk list");

    revwalk
}

fn rewrite_repo_history(repo: &Repository,
                        revwalk: &mut Revwalk,
                        old_id_to_new: &mut HashMap<Oid, Oid>,
                        submodule_dir: &str) {
    let submodule_path = std::path::Path::new(submodule_dir);

    for maybe_oid in revwalk {
        match maybe_oid {
            Ok(oid) => {
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

                // In commits that used to update the submodule, add a parent pointing to
                // appropriate commit in new submodule history
                let mut parent_subtree_ids = std::collections::HashSet::new();
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
                               e.class() == git2::ErrorClass::Tree
                            {
                                continue;
                            } else {
                                panic!("Error getting submodule's subdir from the tree: {:?}", e);
                            };
                        }
                    }
                }

                // Here's a few pictures to help you understand how we figure out if current commit
                // updated the submodule. If we draw a DAG and name submodule states, the following
                // situations will mean that the submodule wasn't updated:
                //
                //     o--o--o--A--
                //                 `,-A
                //      o--o--o--B-
                //
                // or
                //
                //     o--o--o--A--
                //                 `,-B
                //      o--o--o--B-
                //
                // And in the following graphs the submodule was updated:
                //
                //     o--o--o--A--
                //                 `,-C
                //      o--o--o--B-
                //
                // or
                //
                //     o--o--o--o--A--B
                //
                // Put into words, the rule will be "the submodule state in current commit is
                // different from states in all its parents". Or, more formally, the current state
                // doesn't belong to the set of states in parents.
                let submodule_updated: bool = !parent_subtree_ids.contains(&submodule_commit_id);

                // Rewrite the parents if the submodule was updated
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
}

fn checkout_rewritten_history(repo: &Repository, old_id_to_new: &HashMap<Oid, Oid>) {
    let mut checkoutbuilder = git2::build::CheckoutBuilder::new();
    checkoutbuilder.force();

    let head = repo.head().expect("Couldn't obtain repo's HEAD");
    let head_id = head.target().expect("Couldn't resolve repo's HEAD to a commit ID");
    let updated_id = old_id_to_new[&head_id];
    let object = repo.find_object(updated_id, None)
        .expect("Couldn't look up an object at which HEAD points");
    repo.reset(&object, git2::ResetType::Hard, Some(&mut checkoutbuilder))
        .expect("Couldn't run force-reset");
}
