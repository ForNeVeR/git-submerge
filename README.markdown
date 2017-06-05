git-submerge
============

Merge Git submodule into the repo, as if they were never separate at all.



Suppose you have a repo (called "repo") with a submodule (called "sub"). Their
histories might look like this:


    repository  A------C-D-E----G-----K----N
                      ;        ;          ;
                     ;        ;          ;
    submodule       B--------F----H-J---L

(Semicolons are used to distinguish so-called gitlinks from merges.)

After running `git submerge sub` in the root of the repo, the history of the
latter will look like this:

    repository  A----C'-D'-E'---G'-----K'---N'
                 \  / \        / \         /
                  B'   -------F'  H'-J'---L'

In other words, submodule will be replaced by an ordinary directory, and its
history will become part of the repo's history. Moreover, merges will be
structured in the same way as submodule updates in the original history,
preserving the ability to `git-bisect`.

Just as any other kind of history rewriting, this operation changes the hashes
of the commits, so you shouldn't run it on published histories.

TODO: in the graph above, will B' *really* inherit from A, or will it be
a separate history?
