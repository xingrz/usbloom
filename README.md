# USBloom

**Explore your USB devices, down to every endpoint.**

USBloom is a native desktop explorer for the moments when “device
connected” is not enough. Follow a board through its hubs, inspect its
interfaces, and understand what each endpoint does.

- Browse the physical USB tree, with search that keeps parent hubs visible.
- Inspect configurations, interfaces, and alternate settings side by side
  with their decoded class names and endpoint details.
- Read transfer direction, packet sizes, and service intervals without
  translating every field by hand. Hover over endpoints for context.
- Keep up with connected devices, or pause live refresh while inspecting.
- Copy individual device fields or JSON. Save a snapshot and open it later,
  even when the hardware is no longer connected.

USBloom observes devices. Choosing a configuration or alternate setting
in the interface does not change the hardware.

## Try it

The first version is in development. macOS is the primary platform;
Windows and Linux are checked by CI but still need hands-on validation.
There is no signed public installer yet. See
[Contributing](CONTRIBUTING.md#run-and-check) to run a local build.

Select a device on the left, then explore **Interfaces**, **Device details**,
or **Raw data**. Use the pause button beside **Refresh** to stop automatic
updates while inspecting a device. **Save snapshot** keeps the current
capture; **Open** browses one offline, and **Connected devices** returns
to the hardware.

| Shortcut | Action |
| --- | --- |
| ⌘/Ctrl F | Find a device |
| ⌘/Ctrl R | Refresh / return to live devices |
| ⌘/Ctrl S | Save a snapshot |
| ⌘/Ctrl O | Open a snapshot |

Some devices or operating-system permissions prevent full descriptor
reads. USBloom shows unavailable fields rather than guessing. Snapshots
can contain device serial numbers; review them before sharing.

## License

GPL-3.0-or-later. USB inspection is powered by
[cyme](https://github.com/tuna-f1sh/cyme); the native interface uses
[GPUI](https://gpui.rs/) and [GPUI Kit](https://gpui-kit.com/).
