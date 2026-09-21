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
import struct
import subprocess
import tarfile
import tempfile
import zipfile

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


def validate_binary(binary, platform, arch):
    """Reject mislabeled binaries and Windows console-subsystem builds."""
    machines = {"windows": {"x64": 0x8664, "arm64": 0xAA64},
                "linux": {"x64": 62, "arm64": 183}}
    if platform not in machines or arch not in machines[platform]:
        raise ValueError("Unsupported package platform or architecture")
    with binary.open("rb") as stream:
        header = stream.read(64)
        if platform == "linux":
            if (len(header) != 64 or header[:6] != b"\x7fELF\x02\x01" or
                    struct.unpack_from("<H", header, 18)[0] != machines[platform][arch] or
                    struct.unpack_from("<H", header, 16)[0] not in (2, 3)):
                raise ValueError("Expected a matching 64-bit Linux executable")
        else:
            if len(header) != 64 or header[:2] != b"MZ":
                raise ValueError("Expected a Windows executable")
            stream.seek(struct.unpack_from("<I", header, 60)[0])
            pe = stream.read(96)
            if (len(pe) != 96 or pe[:4] != b"PE\0\0" or
                    struct.unpack_from("<H", pe, 4)[0] != machines[platform][arch] or
                    struct.unpack_from("<H", pe, 24)[0] != 0x20B or
                    struct.unpack_from("<H", pe, 92)[0] != 2):
                raise ValueError("Expected a matching 64-bit Windows GUI executable")


def portable_package(tag, output, binary, platform, arch, root=ROOT):
    package_version = version(root)
    validate_tag(tag, package_version)
    validate_binary(binary, platform, arch)
    # These files accompany the exact source archive used by the build job.
    revision = (output / "SOURCE_REVISION").read_text().strip()
    if revision != (root / "SOURCE_REVISION").read_text().strip():
        raise ValueError("Binary source and release source revisions differ")
    name = f"USBloom-{package_version}-{platform}-{arch}"
    with tempfile.TemporaryDirectory() as temp:
        folder = Path(temp) / name
        folder.mkdir()
        executable = folder / ("USBloom.exe" if platform == "windows" else "usbloom")
        shutil.copy2(binary, executable)
        executable.chmod(0o755)
        for filename in ("LICENSE", "THIRD_PARTY_NOTICES.txt", "SOURCE_REVISION"):
            shutil.copy2(output / filename, folder)
        shutil.copy2(root / f"docs/install-{platform}.md", folder / "README.md")
        if platform == "windows":
            destination = output / f"{name}.zip"
            with zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED) as archive:
                for path in sorted(folder.iterdir()):
                    archive.write(path, f"{name}/{path.name}")
        else:
            destination = output / f"{name}.tar.gz"
            def archive_permissions(member):
                member.mode = 0o755 if member.isdir() or member.name == f"{name}/usbloom" else 0o644
                member.uid = member.gid = 0
                member.uname = member.gname = ""
                return member
            with tarfile.open(destination, "w:gz") as archive:
                archive.add(folder, arcname=name, filter=archive_permissions)
        print(destination)


def verify_assets(tag, output):
    package_version = version()
    validate_tag(tag, package_version)
    suffixes = ["macos-arm64.dmg", "source.tar.gz"]
    suffixes += [f"{platform}-{arch}.{extension}"
                 for platform, extension in (("windows", "zip"), ("linux", "tar.gz"))
                 for arch in ("x64", "arm64")]
    required = [f"USBloom-{package_version}-{suffix}" for suffix in suffixes]
    required += ["LICENSE", "THIRD_PARTY_NOTICES.txt", "DEPENDENCIES.json", "SOURCE_REVISION"]
    for filename in required:
        path = output / filename
        if not path.is_file() or path.stat().st_size == 0:
            raise ValueError(f"Missing release asset: {filename}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["version", "check-tag", "bundle", "source", "package",
                                            "portable", "verify-assets", "checksums"])
    parser.add_argument("--tag")
    parser.add_argument("--output", type=Path, default=ROOT / "dist/release")
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/usbloom")
    parser.add_argument("--app", type=Path, default=ROOT / "dist/USBloom.app")
    parser.add_argument("--notices", type=Path)
    parser.add_argument("--platform", choices=["windows", "linux"])
    parser.add_argument("--arch", choices=["x64", "arm64"])
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
    elif args.command == "portable":
        portable_package(args.tag or "", output, args.binary.resolve(), args.platform, args.arch)
    elif args.command == "verify-assets":
        verify_assets(args.tag or "", output)
    else:
        checksums(output)


if __name__ == "__main__":
    main()
