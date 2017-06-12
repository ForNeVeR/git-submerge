extern crate git2;

use git2::Repository;

fn main() {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open the repository in the current directory: {}", e),
    };
}
