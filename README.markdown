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

    Guaranteed not to have conflicts because moving stuff doesn't introduce or
    remove any changes.

2.  Check out `D`. De-init the submodule, remove its directory. Create merge
    commit with parents `B` and `C'`.

    (TODO: manually check if this is really possible and works as described.)

    History at this point:

                         C'-------G'---I'-J'---L'----N'
                          \
                         --D'
                        /
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master

    Guaranteed not to have conflicts because:

    * `C'` is known to contain just submodule's directory;

    * `B` *might* contain submodule's directory, but D must've fixed this,
      otherwise it couldn't add the submodule.


                         C'  -----G3---I3-J3---L3----N3
                          \ /
                         --D'
                        /
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master

    Guaranteed not to have conflicts because:

    * `G'` followed `C'` and changed just the submodule's subdirectory (the
      latter is ensured because we've moved *everything* to submodule's
      subdirectory on step 1);

    * `D'` inherited its submodule's subdirectory from `C'`, so from `G'`'s
      viewpoint, `D'` and `C'` are kinda the same.


                         C'  ---------G3---I3-J3---L3----N3
                          \ /
                         --D'--E'--F'
                        /
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master

    Guaranteed to not have conflicts because:

    * `E` followed `D` and changed everything *but* the submodule's
      subdirectory (the latter is ensured by Git itself—changes to submodule's
      contents would have been committed to the submodule itself, not to the
      containing repo);

    * `D'` contains the same stuff as `D` had, save for submodule's
      subdirectory which `E` doesn't touch.

    Same logic applies with `F` being cherry-picked onto `E'`.

                         C'  ---------G3---I3-J3---L3----N3
                          \ /          \
                         --D'--E'--F'---H'
                        /
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master

    Guaranteed not to have conflicts because submodule's directory hasn't been
    touched by `E'` and `F'` (see proofs above), and `G3` doesn't touch
    anything else (so won't conflict with changes introduced by `E'` and `F'`).


                         C'  ---------G3  -I4-J4---L4----N4
                          \ /          \ /
                         --D'--E'--F'---H'
                        /
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master

    Guaranteed not to have conflicts by the same logic as when we rebased `G3`
    from `C'` onto `D'`.

        ...


                         C'  ---------G3  -I4-J4----L4   -N5
                          \ /          \ /            \ /
                         --D'--E'--F'---H'-------K'----M'----O'
                        /
        repository   A-B----D-E-F----H-------K----M-----O   master
                           ;        ;            ;
                          ;        ;            ;
        submodule        C--------G----I--J----L-----N     master


                         C'  ---------G3  -I4-J4----L4   -N5      sub-master
                          \ /          \ /            \ /
        repository   A-B---D'--E'--F'---H'-------K'----M'----O'   master
