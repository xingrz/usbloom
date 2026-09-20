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

## Device notifications

The app uses nusb's hotplug stream alongside cyme's native acquisition.
On macOS it registers IOKit arrival and termination notifications; Windows
uses Configuration Manager device notifications, and Linux uses udev
netlink events. Register the stream before the initial scan, coalesce
event bursts, and keep descriptor reads on background tasks.

A notification failure is shown with a retry action, without silently
switching to periodic polling. Opening a snapshot invalidates outstanding
scan results and suspends new live scans while notifications stay armed.

## Text selection

GPUI Kit 0.6.4's plain SelectableText needs an explicit redraw subscription.
Each mixed-font run must have an independent scoped element identity.
Long JSON belongs in the bounded read-only Editor viewport; plain-text
selection projects every character and becomes expensive for long inputs.
Keep the drag, idle-repaint, mixed-font, and large-buffer tests when updating
the framework, and repeat the native selection and scrolling checks.
