# Dependency maintenance

GPUI Kit pins a matching GPUI family. Update the Kit version and lockfile
together, then verify native input, focus, scrolling, and all platform builds.

## cyme

The application uses cyme 3.0.2 with its native USB backend. A Cargo
`[patch.crates-io]` override points at a pinned revision of
[xingrz/cyme](https://github.com/xingrz/cyme/commit/4289d2e1f85008431e3ff02cc85474b16533cc72).

This revision is based on upstream v3.0.2 and changes only the
`VariantArray` imports in two files. GPUI enables `strum/derive`; Cargo
unifies that feature with cyme's dependency, which otherwise imports the
same derive macro twice. No profiling or descriptor behavior is changed.

To update, check whether upstream has fixed the imports. If so, remove the
override and update the registry dependency. Otherwise rebase the small
patch onto the chosen upstream tag, pin its full commit, and update
Cargo.lock. Never depend on a moving branch.
