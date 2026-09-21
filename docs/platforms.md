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

The ARM guest smoke checks were performed on 2026-09-21. Manual checks of
window controls passed on both guests. Automated input through Parallels
is unreliable, and physical USB passthrough was not established. These
checks do not validate hardware hotplug or every display scale and window
state. The Ubuntu guest also became unresponsive during hardware testing;
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
controls, not GTK widgets.

Linux client decorations use an application-owned rounded frame. GPUI still
clips child content rectangularly: the root stays transparent, each exposed
surface rounds its own corners, and scroll viewports end above the bottom
corner arcs. New full-bleed surfaces must preserve this layout contract.
Tiled and fullscreen windows use square corners, with resize handles only
on free edges. The platform client inset stays stable during maximize and
restore so the surface does not grow on each transition.

Scrollable panes and the raw descriptor editor share platform-specific
scrollbar geometry. macOS and Windows read GPUI's system auto-hide preference;
Windows uses a narrow indicator that expands on direct interaction. Linux
uses a neutral persistent track unless the desktop supplies an overlay
preference. Tracks stay anchored to the viewport without reflow on hover.
The tree reserves clearance beside its rows; detail panes keep their inset.
These are GPUI controls, not native GTK, Qt, or WinUI widgets.

Linux reads the XDG Settings portal asynchronously and subscribes to changes
before the initial read. Standard `contrast` and `reduced-motion` preferences
keep scrollbar thumbs visible and remove their transition animations,
respectively. The optional `org.gnome.desktop.interface/overlay-scrolling`
backend extension is used when available; it is not assumed to exist on other
desktops. Missing or malformed settings fall back to visible scrollbars and
normal contrast/motion. Portal reads do not change desktop settings. GPUI
already handles the standard color-scheme preference and server-decoration
negotiation. Full GTK/Qt theme rendering and theme-specific scrollbar buttons
are not provided by this integration.
