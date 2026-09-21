# USBloom

**Explore your USB devices, down to every endpoint.**

USBloom is a USB device explorer for macOS, Windows, and Linux.
Follow devices through their hubs and inspect their interfaces,
configurations, and endpoints.

![USBloom showing a USB device tree and serial device details](screenshot.png)

- A searchable device tree that updates as devices connect or disconnect.
- Readable USB classes and endpoint details alongside raw descriptors.
- Snapshots you can save and explore without the hardware connected.

## Install

**macOS (Apple Silicon):** Download the app from
[GitHub Releases](https://github.com/xingrz/usbloom/releases) and drag it
into Applications. If macOS blocks it from opening, run:

```sh
xattr -dr com.apple.quarantine /Applications/USBloom.app
```

**Windows and Linux:** [Build from source](CONTRIBUTING.md#run-and-check).
Packaged releases are currently available for macOS only.

## License

[GPL-3.0-or-later](LICENSE). Built with
[cyme](https://github.com/tuna-f1sh/cyme),
[GPUI](https://gpui.rs/), and [GPUI Kit](https://gpui-kit.com/).
