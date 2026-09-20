# Releasing USBloom

The Release workflow creates a **draft**, never a public release. It runs
on a `v*` tag or can be retried with an existing tag through workflow_dispatch.
Keep a published tag and its assets immutable.

## Prepare

1. Set the version in Cargo.toml and update Cargo.lock. Bundle versions are
   generated from Cargo metadata; there is no separate plist version.
2. Add `docs/releases/v<VERSION>.md`, describing features, installation,
   platform limitations, source availability, and signing status.
3. Review dependency notices when Cargo.lock changes. Published crates may
   omit their workspace's license files. `docs/licenses/index.json` pins
   supplemental upstream notices and their checksums. The source build
   fails if a dependency has neither packaged nor supplemental license text.
4. Run the checks in CONTRIBUTING.md and inspect the native app. Verify
   hotplug, copying, snapshots, and the icon in Finder and the running app.
5. Commit and push, then create and push an annotated tag, for example:

   ```sh
   git tag -a v0.1.0 -m 'USBloom v0.1.0'
   git push origin v0.1.0
   ```

## Automated pipeline

- Reuse the macOS, Windows, and Linux checks.
- Validate the tag against Cargo's version and the checkout commit.
- Archive tracked source and vendor every locked Cargo dependency. Include
  the cyme fork, notices, lockfile, relative Cargo source configuration,
  build instructions, and source revision. Private local captures, build
  output, untracked files, and credentials never enter the source archive.
- Build the Apple Silicon macOS binary **from that source archive**, with
  Cargo's `--frozen` mode and networking disabled for dependency resolution.
- Assemble a fresh `.app`, decode its ICNS to verify integrity, and validate
  its ad-hoc signature. Create a DMG with an Applications shortcut, install
  instructions, license, and notices.
- Compute SHA256 checksums, then attach all files to a draft release. Only
  this final job has permission to write release content. A retry may
  replace draft attachments but refuses to modify a published release.

System fonts are used; no font files are embedded. The macOS package uses
an ad-hoc signature for Apple Silicon execution, not Developer ID signing
or notarization. No signing secrets are required. Windows, Linux, and
Intel macOS installers are outside the first release's scope.

## Review the draft

Download the DMG and source archive from the draft. Verify checksums, the
app icon, drag-to-install, startup, and basic device browsing on a real
Mac. Check the archive's SOURCE_REVISION against the tag. Confirm the
release notes accurately describe platform validation and signing.
Publish the draft only after this review. Keep the matching source asset
available for as long as binaries are distributed.

## Local packaging

Python 3.9+, Rust/rustup, the macOS Command Line Tools, and the system
`iconutil`, `codesign`, `ditto`, and `hdiutil` tools are required. No custom
packaging executables are downloaded. From a clean committed checkout:

```sh
python3 scripts/release.py source --tag v0.1.0
sh scripts/bundle-macos.sh release
python3 scripts/release.py bundle --binary target/release/usbloom \
  --notices dist/release/THIRD_PARTY_NOTICES.txt
python3 scripts/release.py package --tag v0.1.0
python3 scripts/release.py checksums
```

Outputs are in `dist/release/`. To regenerate the icon, run
`swift scripts/render-icon.swift`, then
`iconutil -c icns assets/USBloom.iconset -o assets/USBloom.icns`.
