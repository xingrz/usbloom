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
- Build macOS ARM64 and Windows/Linux x64 and ARM64 binaries **from that
  source archive**, using Cargo's `--frozen` mode with dependency downloads
  disabled.
- Assemble a fresh `.app`, decode its ICNS to verify integrity, and validate
  its ad-hoc signature. Create a DMG with an Applications shortcut, install
  instructions, license, and notices.
- Package Windows as ZIP and Linux as tar.gz, with a single top-level
  folder containing the executable, install notes, license, dependency
  notices, and source revision. Validate executable architecture and the
  Windows GUI subsystem before packaging. Windows statically links its C
  runtime; Linux targets the Ubuntu 24.04 system-library baseline.
- Require all five binary packages and corresponding source before
  computing SHA256 checksums and attaching files to a draft release. Only
  this final job has permission to write release content. A retry may
  replace draft attachments but refuses to modify a published release.

System fonts are used; no font files are embedded. The macOS package uses
an ad-hoc signature for Apple Silicon execution, not Developer ID signing
or notarization. No signing secrets are required. Windows and Linux use
portable archives; Intel macOS packages are not currently produced.

## Review the draft

Download packages and the source archive from the draft. Verify checksums,
archive contents, executable architecture, and source revisions. Check the
macOS app icon and drag-to-install. Run the extracted ARM64 packages in the
Windows and Linux VMs and check startup and basic device browsing. Check
the archive's SOURCE_REVISION against the tag. Confirm the release notes
accurately describe platform validation and signing.
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
