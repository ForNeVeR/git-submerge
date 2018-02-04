git-submerge [![Travis build status][travis-badge]][travis-build] [![Appveyor build status][appveyor-badge]][appveyor-build] [![Andivionian status umbra][andivionian-status-badge]][andivionian-status-link]
============

[travis-badge]: https://travis-ci.org/Minoru/git-submerge.svg?branch=master
[travis-build]: https://travis-ci.org/Minoru/git-submerge
[appveyor-badge]: https://ci.appveyor.com/api/projects/status/2a63wgfyk2utv6f0/branch/master?svg=true
[appveyor-build]: https://ci.appveyor.com/project/Minoru/git-submerge/branch/master
[andivionian-status-badge]: https://img.shields.io/badge/status-enfer-orange.svg
[andivionian-status-link]: https://github.com/ForNeVeR/andivionian-status-classifier##status-enfer-

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

**ATTENTION!** Just as any other kind of history rewriting, `git-submerge`
changes the hashes of the commits, so you shouldn't run it on published
histories. Furthermore, beware of bugs! Run this on a fresh clone of your
repository, and never delete the old history until you're reasonably sure that
the new one is what you expect it to be.

Before using git-submerge, it's recommended to take a look at [a simpler
approach](https://blog.debiania.in.ua/posts/2017-07-06-pulling-submodule-s-history-into-the-main-repository.html).

Dealing with dangling references
================================

It might so happen that `git-submerge` stumbles upon a commit in the main repo
which references a submodule's commit *which doesn't exist*. The reason this
happens is that submodule's history has been rewritten sometime after the
commit to the main repo was made, so now the main repo references something
that is gone.

Rewriting already published histories is generally frowned upon, precisely due
to the problem described above, but it still happens. `git-submerge` provides
you with a couple of flags that you can use to retain as much of your history
as possible. Let's quickly describe what they are, and then we'll take a look at
how one can use them.

The first of those options is `--mapping`, accepting two arguments we'll call
"old commit id" and "new commit id". Whenever `git-submerge` finds a commit in
the main repo that points to "old commit id" in the submodule, it'll pretend
that it sees "new commit id" instead, and will go on with its business.

The second option is `--default-mapping`, accepting one argument we'll call
"default commit id". If `git-submerge` finds a dangling reference which isn't
mentioned in any of the `--mapping`s, it'll use `--default-mapping`. Simple, eh?

Now, as promised, let's look at an example. Suppose you've run `git-submerge`,
and it printed out the following:

```
The repository references the following submodule commits,
but they couldn't be found in the submodule's history:

aaaabbbbccccddddeeeeffff0000111122223333
4444555566667777888899990000aaaabbbbcccc
ddddeeeeffff0000111122223333444455556666

You can use --mapping and --default-mapping options to make
git-submerge replace these commits with some other, still
existing, commits.
```

The best-case scenario for you is that you find a repo that still has these
commits. You can then look at their metadata (commit message, date etc.) and
find the corresponding commits in the submodule's new history.

Another, much more cumbersome, option is to find the aforementioned dangling IDs
in your main repo's history (`git log -S` to the rescue!), then compare with
your submodule's history and simply *guess* at what new commit IDs you could
use.

You can then add a few `--mapping`s, and the problem will be resolved.

The worst-case scenario is that you can't find any trace of the old history, and
guessing didn't help either. In that case, you'll have to create a new commit in
submodule explaining that some of its history has been lost and you can't
recover it. Then you can pass that commit's ID to `--default-mapping`, and the
resulting history will at least have an explanation of why some commits are
broken.

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

Useful tips
===========

* When viewing the rewritten history with `git log --patch`, add `-m`, `-c` or
    `--cc` option; they all enable diffs for merge commits (with slightly
    different presentation—just pick the one you like). The reason this is
    important is that your original history might have had commits where the
    submodule is updated *and* some changes are made; now that such commits are
    turned into merges, Git assumes that file changes were merge conflict
    resolutions, and hides them from the diffs.
