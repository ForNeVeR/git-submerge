git-submerge [![Travis build status][travis-badge]][travis-build] [![Appveyor build status][appveyor-badge]][appveyor-build] [![Andivionian status umbra][status-umbra-badge]][andivionian-status-umbra]
============

[travis-badge]: https://travis-ci.org/Minoru/git-submerge.svg?branch=master
[travis-build]: https://travis-ci.org/Minoru/git-submerge
[appveyor-badge]: https://ci.appveyor.com/api/projects/status/2a63wgfyk2utv6f0/branch/master?svg=true
[appveyor-build]: https://ci.appveyor.com/project/Minoru/git-submerge/branch/master
[status-umbra-badge]: https://img.shields.io/badge/status-enfer-orange.svg
[andivionian-status-umbra]: https://github.com/ForNeVeR/andivionian-status-classifier#status-umbra-

Suppose you have a repo with a submodule. Their collective history might look
like this:

    repository   A-B----D-E-F----H-----K----M---O   master
                       ;        ;          ;
                      ;        ;          ;
    submodule        C--------G----I-J---L----N     master

(Semicolons are gitlinks; we'll use slashes for merges.)

After running `git submerge submodule`, the history will look like this:

    repository   A-B---D'-E'-F'--H'--------K'--M'----O'   master
                      /         /             /
                     C---------G'----I'-J'---L'---N'      sub-master

The following things happened:

* submodule got replaced by an ordinary directory;

* submodule's history became part of the repo's history;

* where submodule updates were previously (commits D, H, M), we now have merge
  commits;

* the yet-unmerged "tail" of the submodule history (commit N) is given its own
  branch so you can merge it yourself later (#18).

Just as any other kind of history rewriting, this operation changes the hashes
of the commits, so you shouldn't run it on published histories.

Building
========

git-submerge requires rustc 1.15+ and cargo 0.16+, so you might need to update
your build environment first:

```console
$ rustup update
```

NixOS users can use the Nix shell; it'll fetch Rust nightly:

```console
$ nix-shell
```

After that, it's the usual jazz:

```console
$ cargo build
```

Testing
=======

To check that your build behaves the way the developers expect, do the following:

1. Prepare a directory for your tests:

    ```console
    $ mkdir /tmp/git-submerge-testbed
    ```

2. Create the submodule repo:

    ```console
    $ cd /tmp/git-submerge-testbed
    # Assuming you have git-submerge cloned to /home/user/git-submerge
    $ git clone /home/user/git-submerge sub
    # ...otherwise
    $ git clone https://github.com/Minoru/git-submerge.git sub
    $ cd sub
    $ git reset --hard poc-submodule
    ```

3. Create the main repo:

    ```console
    $ cd /tmp/git-submerge-testbed
    # Assuming you have git-submerge cloned to /home/user/git-submerge
    $ git clone /home/user/git-submerge repo
    # ...otherwise
    $ git clone https://github.com/Minoru/git-submerge.git repo
    $ cd repo
    $ git reset --hard poc-repo
    # Removing upstream remote so that `git submodule` looks for
    # submodule repo in our testbed, not in the place we cloned from
    $ git remote rm origin
    $ git submodule update --init
    ```

4. Run git-submerge:

    ```console
    # This assumes you've updated your path like so:
    # $ export PATH=/home/user/git-submerge/target/debug/:$PATH
    # Alternatively, you can use full path instead of "git submerge".
    $ git submerge sub
    ```

5. Check the result:

    ```console
    $ git fast-export master > /home/user/git-submerge/test/expected.stream
    $ cd /home/user/git-submerge/test
    $ git diff expected.stream
    ```

    If everything went well, `git diff` shouldn't find any differences, and
    there will be no output.

    Don't forget to clean up afterwards!

    ```console
    $ git checkout expected.stream
    ```
