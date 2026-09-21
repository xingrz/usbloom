"""Regression checks for release identity and distribution integrity."""
import hashlib
import struct
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile
from pathlib import Path

import release


class ReleaseTests(unittest.TestCase):
    def binary(self, path, platform, arch, subsystem=2):
        header = bytearray(256)
        if platform == "linux":
            header[:6] = b"\x7fELF\x02\x01"
            struct.pack_into("<HH", header, 16, 3, 62 if arch == "x64" else 183)
        else:
            header[:2] = b"MZ"
            struct.pack_into("<I", header, 60, 64)
            header[64:68] = b"PE\0\0"
            struct.pack_into("<H", header, 68, 0x8664 if arch == "x64" else 0xAA64)
            struct.pack_into("<H", header, 88, 0x20B)
            struct.pack_into("<H", header, 156, subsystem)
        path.write_bytes(header)

    def test_tag_must_match_the_package_version(self):
        release.validate_tag("v0.1.0", "0.1.0")
        release.validate_tag("v0.2.0-rc.1", "0.2.0-rc.1")
        for tag in ("v0.2.0", "master", "0.1.0", "v0.1.0/extra", "v0.1.0\n"):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                release.validate_tag(tag, "0.1.0")

    def test_checksums_cover_assets_and_do_not_hash_themselves(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "app.dmg").write_bytes(b"application")
            (root / "source.tar.gz").write_bytes(b"source")
            release.checksums(root)
            original = (root / "SHA256SUMS").read_text()
            release.checksums(root)
            self.assertEqual((root / "SHA256SUMS").read_text(), original)
            self.assertEqual(len(original.splitlines()), 2)
            for line in original.splitlines():
                digest, name = line.split("  ")
                self.assertEqual(digest, hashlib.sha256((root / name).read_bytes()).hexdigest())
            (root / "app.dmg").write_bytes(b"changed")
            release.checksums(root)
            self.assertNotEqual((root / "SHA256SUMS").read_text(), original)

    def test_mislabeled_and_console_binaries_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            binary = Path(folder) / "binary"
            for platform in ("linux", "windows"):
                for arch in ("x64", "arm64"):
                    with self.subTest(platform=platform, arch=arch):
                        self.binary(binary, platform, arch)
                        release.validate_binary(binary, platform, arch)
                        wrong_arch = "arm64" if arch == "x64" else "x64"
                        with self.assertRaises(ValueError):
                            release.validate_binary(binary, platform, wrong_arch)
            self.binary(binary, "windows", "x64", subsystem=3)
            with self.assertRaises(ValueError):
                release.validate_binary(binary, "windows", "x64")
            binary.write_bytes(b"truncated")
            for platform in ("linux", "windows"):
                with self.assertRaises(ValueError):
                    release.validate_binary(binary, platform, "x64")

    @patch("release.version", return_value="0.1.1")
    def test_portable_archives_include_source_identity_and_notices(self, _version):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            output = root / "release"
            output.mkdir()
            (root / "docs").mkdir()
            (root / "SOURCE_REVISION").write_text("abc123\n")
            for filename in ("LICENSE", "THIRD_PARTY_NOTICES.txt", "SOURCE_REVISION"):
                (output / filename).write_text("abc123\n")
            for platform, extension, exe in (("windows", "zip", "USBloom.exe"),
                                             ("linux", "tar.gz", "usbloom")):
                (root / f"docs/install-{platform}.md").write_text("Install\n")
                binary = root / "binary"
                self.binary(binary, platform, "arm64")
                release.portable_package("v0.1.1", output, binary, platform, "arm64", root)
                name = f"USBloom-0.1.1-{platform}-arm64"
                expected = {f"{name}/{filename}" for filename in
                            (exe, "README.md", "LICENSE", "THIRD_PARTY_NOTICES.txt", "SOURCE_REVISION")}
                if platform == "windows":
                    with zipfile.ZipFile(output / f"{name}.{extension}") as archive:
                        self.assertEqual(set(archive.namelist()), expected)
                        self.assertEqual(archive.read(f"{name}/{exe}"), binary.read_bytes())
                        self.assertEqual(archive.read(f"{name}/SOURCE_REVISION"),
                                         (output / "SOURCE_REVISION").read_bytes())
                else:
                    with tarfile.open(output / f"{name}.{extension}") as archive:
                        self.assertEqual({m.name for m in archive if m.isfile()}, expected)
                        self.assertEqual(archive.getmember(f"{name}/{exe}").mode & 0o777, 0o755)
                        self.assertEqual(archive.extractfile(f"{name}/{exe}").read(), binary.read_bytes())
                (output / "SOURCE_REVISION").write_text("different\n")
                with self.assertRaises(ValueError):
                    release.portable_package("v0.1.1", output, binary, platform, "arm64", root)
                (output / "SOURCE_REVISION").write_text("abc123\n")

    @patch("release.version", return_value="0.1.1")
    def test_draft_requires_every_platform_package(self, _version):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            assets = ["macos-arm64.dmg", "source.tar.gz", "windows-x64.zip",
                      "windows-arm64.zip", "linux-x64.tar.gz", "linux-arm64.tar.gz"]
            files = [f"USBloom-0.1.1-{name}" for name in assets]
            files += ["LICENSE", "THIRD_PARTY_NOTICES.txt", "DEPENDENCIES.json", "SOURCE_REVISION"]
            for filename in files:
                (root / filename).write_bytes(b"asset")
            release.verify_assets("v0.1.1", root)
            for filename in files:
                with self.subTest(filename=filename):
                    (root / filename).write_bytes(b"")
                    with self.assertRaises(ValueError):
                        release.verify_assets("v0.1.1", root)
                    (root / filename).write_bytes(b"asset")


if __name__ == "__main__":
    unittest.main()
