git-submerge [![Build Status](https://travis-ci.org/Minoru/git-submerge.svg?branch=master)](https://travis-ci.org/Minoru/git-submerge) [![Build status](https://ci.appveyor.com/api/projects/status/2a63wgfyk2utv6f0/branch/master?svg=true)](https://ci.appveyor.com/project/Minoru/git-submerge/branch/master)
============

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

Build instructions
==================

Requires rustc 1.15+ and cargo 0.16+.

Testing
=======

To check that your build behaves the way the developers expect, do the following:

1. Prepare a directory for your tests:

    ```console
    $ mkdir /dev/shm/git-submerge-testbed
    ```

2. Create the submodule repo:

    ```console
    $ cd /dev/shm/git-submerge-testbed
    # Assuming you have git-submerge cloned to /home/user/git-submerge
    $ git clone /home/user/git-submerge sub
    # ...otherwise
    $ git clone https://github.com/Minoru/git-submerge.git sub
    $ cd sub
    $ git reset --hard poc-submodule
    ```

3. Create the main repo:

    ```console
    $ cd /dev/shm/git-submerge-testbed
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
    $ diff -ruN /home/user/git-submerge/test/expected.stream \
        <(git fast-export master)
    ```

    If everything went well, `diff` shouldn't find any differences, and there
    will be no output.
