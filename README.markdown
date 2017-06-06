git-submerge
============

Merge Git submodule into the repo, as if they've never been separate at all.



Suppose you have a repo with a submodule. Their collective history might look
like this:

    repository   A-B----D-E-F----H-----L----N---P   master
                       ;        ;          ;
                      ;        ;          ;
    submodule        C--------G----J-K---M----O     master

(Semicolons are gitlinks; we'll use slashes for merges.)

After running `git submerge sub`, the history will look like this:

    repository   A-B---D'-E'-F'--H'--------L'--N'----P'   master
                    \ / \       / \           /
                     C'  ------G'  --J'-K'---M'---O'      sub-master

The following things happened:

* submodule got replaced by an ordinary directory;

* submodule's history became part of the repo's history;

* where submodule updates were previously (commits D, H, N), we now have merge
  commits;

* the yet-unmerged "tail" of the submodule history (commit O) is given its own
  branch so you can merge it yourself later.

Just as any other kind of history rewriting, this operation changes the hashes
of the commits, so you shouldn't run it on published histories.
