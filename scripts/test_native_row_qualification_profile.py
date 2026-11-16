"""Behavioral checks for bounded selection from verified source profiles."""
import json
from pathlib import Path
import tempfile
import unittest

from native_row_qualification_profile import mantle_source_paths


class SourceProfileSelectionTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="mantle-native-profile-")
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "profile.json"
        self.source = "bootstrap/gcc-4.0.ncl"
        self.digest = "a" * 64
        self.manifest = "b" * 64
        self.profile = {
            "format": "mantle-source-bundle-v1", "version": 1,
            "store_prefix": "/mantle/store", "roots": [],
            "records": [{
                "kind": "local-path",
                "metadata": {
                    "bootstrap_profile_class": "mantle-source",
                    "bootstrap_profile_mode": "source-built-fixed-point",
                },
                "files": [{"path": self.source, "file_type": "regular",
                           "blake3": self.digest}],
            }],
            "manifest_blake3": self.manifest,
            "non_claim": "No execution is established by this fixture.",
        }

    def select(self, manifest=None):
        self.path.write_text(json.dumps(self.profile), encoding="utf-8")
        return mantle_source_paths(self.path, manifest or self.manifest, {self.source})

    def test_escaped_metadata_and_large_unselected_payload_preserve_selection(self):
        escaped = '\n"quoted"\\backslash\t\u03bb'
        self.profile["records"][0]["ignored"] = ("x" * 1_048_573 + escaped) * 2
        self.profile["non_claim"] += escaped
        self.assertEqual(self.select(), {self.source: self.digest})

    def test_wrong_profile_identity_and_missing_source_fail(self):
        with self.assertRaisesRegex(ValueError, "identity/prefix mismatch"):
            self.select("c" * 64)
        self.profile["records"][0]["files"].clear()
        with self.assertRaisesRegex(ValueError, "lacks exact Mantle row source entries"):
            self.select()


if __name__ == "__main__":
    unittest.main()
