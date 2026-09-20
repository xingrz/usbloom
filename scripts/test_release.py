"""Regression checks for release identity and distribution integrity."""
import hashlib
import tempfile
import unittest
from pathlib import Path

import release


class ReleaseTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
