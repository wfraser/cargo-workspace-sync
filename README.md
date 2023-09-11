cargo workspace-sync
--------------------

Cargo workspaces are awesome. They allow a number of packages to share dependencies, making for
faster builds and making it easy to keep dependencies in sync between them.

Unfortunately, the feature is rather intrusive. Once the top-level Cargo.toml is added with the
`[workspace]` section, all dependency info for all members goes in a workspace-level lockfile and
any member-level lockfile is ignored. This makes it impossible to distribute the workspace members
individually, which might be a problem in some situations (like a mix of open-source and heavily
internal projects).

`cargo workspace-sync` is a hack that fixes this limitation by allowing you to also maintain
workspace member-specific lockfiles in addition to the main workspace root one. This way you can
even keep the workspace members as separate git repositories and join them as git submodules, and
they can all be built individually, but still makes cross-member dependency synchronization easy.

It works by temporarily removing the root-level workspace, copying the root lockfile into each
member, and then letting Cargo remove redundant or extraneous dependencies which don't apply to that
package, then restoring the workspace.

Usage:

```
$ cargo update ...args  # make some changes or whatever
$ git commit -m "cargo update" Cargo.lock
$ cargo workspace-sync
```
And now you should see changes to all workspace members' Cargo.lock files which were affected by the
root-level changes.