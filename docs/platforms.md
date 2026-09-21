# Platform validation

Build support and runtime validation are separate. The first distributed
package targets Apple Silicon macOS; Windows and Linux remain experimental.

| Target | CI | Runtime evidence |
| --- | --- | --- |
| macOS ARM64 | Format, lint, tests, build | Native GUI, real USB descriptors, hotplug, copying, snapshots, appearance, and app packaging |
| Windows x64 | Format, lint, tests, build | Not yet tested interactively |
| Windows ARM64 | Format, lint, tests, release build | Starts on Windows 11 in Parallels; renders icons, titlebar controls, and virtual USB device details |
| Linux x64 | Format, lint, tests, build | Not yet tested interactively |
| Linux ARM64 | Format, lint, tests, release build | Starts on Ubuntu 24.04 in Parallels and lists virtual USB devices; descriptor access can be denied |

The ARM guest smoke checks were performed on 2026-09-21. Physical USB
passthrough and reliable guest input were not established, so these checks
do not validate hardware hotplug, titlebar hit targets, or every window
state on Windows or Linux. The Ubuntu guest also became unresponsive;
the cause has not been established.

## Portable validation builds

ARM CI retains release binaries for seven days for local VM testing.
Use `cargo build --release --locked` when moving a binary between machines.
GPUI Kit's full icon catalog is embedded in release builds; a debug binary
can depend on asset files in the original build environment.

## Window checks

Check a fresh launch, activation, dragging, resizing, minimizing,
maximizing/restoring, and fullscreen transitions. Check the existing
display scale and another available scale when practical. Toolbar content
and window controls must remain aligned, clickable, and clear of each
other; toolbar actions must not initiate window dragging.

macOS applies its traffic-light placement after the initial native layout.
Windows uses system caption glyphs in 46-by-32 logical-pixel hit regions
anchored to the top right. GPUI maps these regions to native window actions,
including the maximize snap flyout. The integrated toolbar is 48 pixels high.
See Microsoft's [titlebar design guidance](https://learn.microsoft.com/en-us/windows/apps/design/basics/titlebar-design).

Linux requests server decorations and omits duplicate client controls when
the compositor supplies them. GNOME on Wayland requires client decorations;
its fallback uses circular controls in a 48-pixel headerbar. Other client
decoration environments retain Kit's controls. These are application-drawn
controls, not GTK widgets. Kit's current Linux frame remains square because
GPUI does not provide rounded content clipping; rounded GNOME window frames
are not yet reproduced.
