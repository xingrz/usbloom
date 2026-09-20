#!/usr/bin/env python3
"""Build distribution assets from locked, committed source using system tools."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
APP_ID = "me.xingrz.usbloom"


def run(*args, cwd=ROOT):
    return subprocess.check_output(args, cwd=cwd, text=True)


def metadata(root=ROOT, no_deps=False):
    args = ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"]
    if no_deps:
        args.append("--no-deps")
    return json.loads(run(*args, cwd=root))


def version(root=ROOT):
    return next(p["version"] for p in metadata(root, True)["packages"]
                if p["name"] == "usbloom")


def validate_tag(tag, package_version):
    if not re.fullmatch(r"v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", tag):
        raise ValueError("Expected a version tag such as v0.1.0")
    if tag != "v" + package_version:
        raise ValueError(f"Tag {tag} does not match Cargo version {package_version}")


def bundle(binary, destination, notices=None):
    """Replace the entire generated bundle, including Finder-visible metadata."""
    destination = destination.resolve()
    if destination.suffix != ".app":
        raise ValueError("Bundle destination must end in .app")
    if destination.exists():
        with (destination / "Contents/Info.plist").open("rb") as f:
            if plistlib.load(f).get("CFBundleIdentifier") != APP_ID:
                raise ValueError("Refusing to replace an unrelated application")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=destination.parent) as temp:
        staged = Path(temp) / destination.name
        contents = staged / "Contents"
        resources = contents / "Resources"
        resources.mkdir(parents=True)
        (contents / "MacOS").mkdir()
        shutil.copy2(binary, contents / "MacOS/USBloom")
        shutil.copy2(ROOT / "assets/USBloom.icns", resources / "USBloom.icns")
        shutil.copy2(ROOT / "LICENSE", resources / "LICENSE")
        if notices:
            shutil.copy2(notices, resources / "THIRD_PARTY_NOTICES.txt")
        info = {
            "CFBundleExecutable": "USBloom",
            "CFBundleIdentifier": APP_ID,
            "CFBundleIconFile": "USBloom.icns",
            "CFBundleName": "USBloom",
            "CFBundleDisplayName": "USBloom",
            "CFBundlePackageType": "APPL",
            "CFBundleShortVersionString": version(),
            "CFBundleVersion": version().split("-")[0],
            "NSHighResolutionCapable": True,
            "LSMinimumSystemVersion": "12.0",
        }
        with (contents / "Info.plist").open("wb") as f:
            plistlib.dump(info, f)
        # Decode every representation before distributing a malformed icon.
        run("iconutil", "-c", "iconset", str(resources / "USBloom.icns"),
            "-o", str(Path(temp) / "verify.iconset"))
        # Apple Silicon requires an ad-hoc code signature. This is not a
        # Developer ID signature or notarization and needs no credentials.
        run("codesign", "--force", "--sign", "-", "--identifier", APP_ID, str(staged))
        run("codesign", "--verify", "--strict", str(staged))
        if destination.exists():
            destination.rename(Path(temp) / "previous.app")
        staged.rename(destination)
        os.utime(destination, None)
    print(destination)


def license_files(root):
    return sorted(p for p in root.rglob("*") if p.is_file() and
                  p.name.lower().startswith(("license", "copying", "notice", "unlicense", "ofl")))


def notices(root, output):
    extras_root = root / "docs/licenses"
    extras = {(e["name"], e["version"]): e["files"]
              for e in json.loads((extras_root / "index.json").read_text())}
    packages = sorted((p for p in metadata(root)["packages"] if p["source"]),
                      key=lambda p: (p["name"], p["version"]))
    inventory = []
    chunks = ["USBloom third-party notices\n\n"
              "This inventory includes build, test, and other-platform dependencies\n"
              "contained in the corresponding source archive, as well as runtime code.\n"
              "The application uses system fonts; no font files are bundled.\n\n"]
    for p in packages:
        package_root = Path(p["manifest_path"]).parent
        files = license_files(package_root)
        supplemental = extras.get((p["name"], p["version"]), [])
        if not files and not supplemental:
            raise ValueError(f"Missing license text: {p['name']} {p['version']}")
        inventory.append({k: p[k] for k in
                          ("name", "version", "source", "license", "repository", "authors")})
        chunks.append(f"\n{'=' * 72}\n{p['name']} {p['version']}\n"
                      f"License: {p['license'] or p['license_file']}\n"
                      f"Repository: {p['repository'] or '(see source)'}\n"
                      f"Authors: {', '.join(p['authors'])}\n")
        attributions = sorted(p for p in package_root.iterdir() if p.is_file() and
                              p.name.lower().startswith(("authors", "copyright")))
        for path in files + attributions:
            chunks.append(f"\n--- {path.relative_to(package_root)} ---\n" +
                          path.read_text(errors="replace") + "\n")
        for entry in supplemental:
            data = (extras_root / entry["file"]).read_bytes()
            if hashlib.sha256(data).hexdigest() != entry["sha256"]:
                raise ValueError(f"Notice checksum mismatch: {entry['file']}")
            kind = entry.get("kind", "upstream-notice")
            chunks.append(f"\n--- {kind}: {entry['url']} ---\n" + data.decode() + "\n")
    output.write_text("".join(chunks))
    output.with_name("DEPENDENCIES.json").write_text(json.dumps(inventory, indent=2) + "\n")


def source(tag, output):
    validate_tag(tag, version())
    run("git", "diff", "--exit-code", "HEAD", "--")
    revision = run("git", "rev-parse", "HEAD").strip()
    output.mkdir(parents=True, exist_ok=True)
    name = f"USBloom-{version()}-source"
    with tempfile.TemporaryDirectory(prefix="usbloom-source-") as temp:
        root = Path(temp) / name
        root.mkdir()
        archive = Path(temp) / "tracked.tar"
        run("git", "archive", "--format=tar", "--output", str(archive), "HEAD")
        run("tar", "-xf", str(archive), "-C", str(root))
        # Source paths remain relative so the archive can be built elsewhere.
        config = run("cargo", "vendor", "--locked", "--versioned-dirs", "vendor", cwd=root)
        (root / ".cargo").mkdir(exist_ok=True)
        (root / ".cargo/config.toml").write_text(config)
        (root / "SOURCE_REVISION").write_text(revision + "\n")
        (root / "BUILDING.txt").write_text(
            "Install the toolchain in rust-toolchain.toml and platform SDKs.\n"
            "All locked Cargo dependencies, including the cyme patch, are in vendor/.\n"
            "Build without fetching dependencies: cargo build --release --frozen\n"
            "On macOS: CARGO_NET_OFFLINE=true sh scripts/bundle-macos.sh release\n"
            "See CONTRIBUTING.md and docs/releasing.md for tools and packaging.\n")
        notices(root, output / "THIRD_PARTY_NOTICES.txt")
        shutil.copy2(output / "THIRD_PARTY_NOTICES.txt", root)
        shutil.copy2(output / "DEPENDENCIES.json", root)
        # Resolution must succeed using only the supplied vendor directory.
        run("cargo", "metadata", "--frozen", "--format-version", "1", cwd=root)
        run("tar", "-czf", str(output / (name + ".tar.gz")), "-C", temp, name)
    shutil.copy2(ROOT / "LICENSE", output / "LICENSE")
    (output / "SOURCE_REVISION").write_text(revision + "\n")


def package(tag, output, app):
    validate_tag(tag, version())
    if run("uname", "-m").strip() != "arm64":
        raise ValueError("The first macOS package targets Apple Silicon only")
    run("lipo", str(app / "Contents/MacOS/USBloom"), "-verify_arch", "arm64")
    with tempfile.TemporaryDirectory() as temp:
        folder = Path(temp) / "USBloom"
        folder.mkdir()
        run("ditto", str(app), str(folder / "USBloom.app"))
        for name in ("LICENSE", "THIRD_PARTY_NOTICES.txt", "SOURCE_REVISION"):
            shutil.copy2(output / name, folder)
        shutil.copy2(ROOT / "docs/install-macos.md", folder / "README.md")
        # A /Applications link supports the usual drag-to-install workflow.
        (folder / "Applications").symlink_to("/Applications")
        run("hdiutil", "create", "-ov", "-format", "UDZO", "-volname", "USBloom",
            "-srcfolder", str(folder), str(output / f"USBloom-{version()}-macos-arm64.dmg"))


def checksums(output):
    files = sorted(p for p in output.iterdir() if p.is_file() and p.name != "SHA256SUMS")
    lines = []
    for path in files:
        digest = hashlib.sha256()
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
        lines.append(f"{digest.hexdigest()}  {path.name}\n")
    (output / "SHA256SUMS").write_text("".join(lines))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["version", "check-tag", "bundle", "source", "package", "checksums"])
    parser.add_argument("--tag")
    parser.add_argument("--output", type=Path, default=ROOT / "dist/release")
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/usbloom")
    parser.add_argument("--app", type=Path, default=ROOT / "dist/USBloom.app")
    parser.add_argument("--notices", type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    if args.command == "version":
        print(version())
    elif args.command == "check-tag":
        validate_tag(args.tag or "", version())
    elif args.command == "bundle":
        bundle(args.binary, args.app, args.notices)
    elif args.command == "source":
        source(args.tag or "", output)
    elif args.command == "package":
        package(args.tag or "", output, args.app.resolve())
    else:
        checksums(output)


if __name__ == "__main__":
    main()
