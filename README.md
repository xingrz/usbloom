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

Download your platform's package from
[GitHub Releases](https://github.com/xingrz/usbloom/releases).

| Platform | Package | Install |
| --- | --- | --- |
| macOS · Apple Silicon | DMG | Drag USBloom into Applications |
| Windows · x64 / ARM64 | ZIP | Extract and open USBloom.exe |
| Linux · x64 / ARM64 | tar.gz | Extract and run usbloom |

If macOS blocks the app from opening:

```sh
xattr -dr com.apple.quarantine /Applications/USBloom.app
```

See [Linux requirements](docs/install-linux.md) if needed.

## License

[GPL-3.0-or-later](LICENSE). Built with
[cyme](https://github.com/tuna-f1sh/cyme),
[GPUI](https://gpui.rs/), and [GPUI Kit](https://gpui-kit.com/).
