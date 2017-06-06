git-submerge
============

Merge Git submodule into the repo, as if they've never been separate at all.



Suppose you have a repo with a submodule. Their collective history might look
like this:

    repository   A-B----D-E-F----H-----K----M---O   master
                       ;        ;          ;
                      ;        ;          ;
    submodule        C--------G----I-J---L----N     master

(Semicolons are gitlinks; we'll use slashes for merges.)

After running `git submerge submodule`, the history will look like this:

    repository   A-B---D'-E'-F'--H'--------K'--M'----O'   master
                    \ / \       / \           /
                     C'  ------G'  --I'-J'---L'---N'      sub-master

The following things happened:

* submodule got replaced by an ordinary directory;

* submodule's history became part of the repo's history;

* where submodule updates were previously (commits D, H, M), we now have merge
  commits;

* the yet-unmerged "tail" of the submodule history (commit N) is given its own
  branch so you can merge it yourself later.

Just as any other kind of history rewriting, this operation changes the hashes
of the commits, so you shouldn't run it on published histories.


How it works
============

Let's take another look at the history we saw before:

    repository   A-B----D-E-F----H-----K----M---O   master
                       ;        ;          ;
                      ;        ;          ;
    submodule        C--------G----I-J---L----N     master

Suppose the submodule is checked out into a directory called "sub".

`git submerge` proceeds in the following way:

1.  Rewrites submodule history, moving all files into a directory called "sub".
    This yields the following history:

        C'--G'--I'--J'--L'--N'

2.  Starting with `O`, walks back into history to find the commit that added
    the submodule (i.e. `D`), then takes its ancestor (`B`).

    If there are more than one ancestor, the tool will ask user which commit
    they want to use.

3.  Rewritten history from step 1 is rebased onto the commit from step 2. At
    this point it's helpful to start looking at the whole history:

                         C"-------G"---I"-J"---L"----N"
                        /
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master

4.  Old and new submodule histories (`C..N` and `C"..N"`, respectively) are
    walked in lockstep to learn the correspondence between old and new commit
    IDs. (We humans see the link between `C` and `C"`; it's tougher for
    computers who deal with SHA-1 hashes.)

5.  Check out `D`. De-init the submodule, remove its directory. Create merge
    commit with parents `B` and `C"`.

    We now have commit `D'`, which we'll call "tip". We'll also call `C"` as
    "current submodule commit".

    (TODO: manually check if this is really possible and works as described.)

    History at this point:

                         C"-------G"---I"-J"---L"----N"
                        / \
                       | --D'
                       |/
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master


                         C"  -----G3---I3-J3---L3----N3
                        / \ /
                       | --D'
                       |/
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master


                         C"  ---------G3---I3-J3---L3----N3
                        / \ /
                       | --D'--E'--F'
                       |/
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master


                         C"  ---------G3---I3-J3---L3----N3
                        / \ /          \
                       | --D'--E'--F'---H'
                       |/
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master


                         C"  ---------G3  -I4-J4---L4----N4
                        / \ /          \ /
                       | --D'--E'--F'---H'
                       |/
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master


        ...


                         C"  ---------G3  -I4-J4----L4   -N5
                        / \ /          \ /            \ /
                       | --D'--E'--F'---H'-------K'----M'----O'
                       |/
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master


                         C"  ---------G3  -I4-J4----L4   -N5      sub-master
                        / \ /          \ /            \ /
        repository   A-B---D'--E'--F'---H'-------K'----M'----O'   master
