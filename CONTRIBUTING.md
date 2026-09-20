# Contributing to USBloom

USBloom is an early native desktop application. Keep changes focused and
include enough evidence for someone else to reproduce the result. Read
[AGENTS.md](AGENTS.md) for architecture and agent workflow requirements.

## Run and check

Install Rust with rustup; the repository selects its toolchain. On macOS,
install the Command Line Tools with `xcode-select --install`. GPUI Kit uses
runtime Metal shaders, so the full Xcode application is not required.

```sh
cargo run --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

On Ubuntu 24.04, install the packages listed in
[the build workflow](.github/workflows/build.yml). Windows builds require
the MSVC toolchain and Windows SDK. CI checks macOS ARM64 and both x64 and
ARM64 Windows/Linux targets. See [platform validation](docs/platforms.md)
for the runtime evidence and remaining gaps.

Create a local macOS bundle with:

```sh
sh scripts/bundle-macos.sh
# Optimized build:
sh scripts/bundle-macos.sh release
```

The bundle is written to `dist/USBloom.app` with a stable identifier. It
uses an ad-hoc signature for local development, not notarization or an
Apple distribution identity. The app icon is generated from
`scripts/render-icon.swift`; regenerate it with Swift followed by
`iconutil -c icns assets/USBloom.iconset -o assets/USBloom.icns`.

## Verify a change

Use synthetic devices for automated tests. Exercise nested hubs, missing
fields, alternate settings with different endpoint lists, invalid imports,
and interrupted or partial reads. Never commit captures of real hardware.

For UI changes, run the actual application. Check search, hub expansion,
selection across refresh, scrolling, alternate selection, copying values,
and a save/open roundtrip. A build does not establish GUI compatibility.
Do not reset devices or change their configuration to test the viewer.

Snapshots include serial numbers and host/device details. Review and
sanitize them before sharing. USBloom keeps them local and has no upload
or telemetry feature.

## Dependencies and distribution

Keep Cargo.lock committed. See [dependency maintenance](docs/dependencies.md)
for the pinned cyme fork and the conditions for removing its patches.

Before publishing binaries, audit bundled dependency and font notices,
include the GPL license, and provide the exact corresponding source,
lockfile, build scripts, and patched dependency sources. A reproducible
source bundle can include `cargo vendor --locked` output and the Cargo
source replacement configuration. The first macOS release uses an ad-hoc
signature without notarization; state that clearly in installation
instructions. Developer ID signing and notarization are deferred. See
[the release guide](docs/releasing.md) for the automated draft-release
pipeline and review steps.

Use English Conventional Commits with a short subject and a short body
explaining why. Wrap each line at 75 characters. Fix nearby unpublished
mistakes by amending or autosquashing, and preserve the repository's Git
identity.
