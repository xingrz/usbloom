# Dependency maintenance

GPUI Kit pins a matching GPUI family. Update the Kit version and lockfile
together, then verify native input, focus, scrolling, and all platform builds.

## cyme

The application uses cyme 3.0.2 with its native USB backend. A Cargo
`[patch.crates-io]` override points at a pinned revision of
[xingrz/cyme](https://github.com/xingrz/cyme/commit/71bfc962c31f7323135bd918b7e46eff6e010edd).

This revision is based on upstream v3.0.2 and corrects the
`VariantArray` imports in two files. GPUI enables `strum/derive`; Cargo
unifies that feature with cyme's dependency, which otherwise imports the
same derive macro twice.

It also serializes negotiated USB speeds as named enum variants. Upstream
uses untagged unit variants, which serialize as null and lose speed data
when reopening a profile. A roundtrip test covers every speed variant.

To update, check whether upstream has fixed both issues. If so, remove the
override and update the registry dependency. Otherwise rebase the small
patch onto the chosen upstream tag, pin its full commit, and update
Cargo.lock. Never depend on a moving branch.
