"""Behavioral regressions for the bounded lock-vendor source archive."""

import importlib.util
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/vendor-source-bundle.py"
spec = importlib.util.spec_from_file_location("vendor_source_bundle", SCRIPT)
assert spec is not None and spec.loader is not None
bundle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bundle)


class SourceBundleGitCheckoutTest(unittest.TestCase):
    def test_exported_checkout_keeps_empty_git_directories_and_exact_revision(self):
        with tempfile.TemporaryDirectory(prefix="vendor-bundle-git-") as temporary:
            root = Path(temporary)
            checkout = root / "checkout"
            subprocess.run(["git", "init", "-q", "--initial-branch=main", str(checkout)], check=True)
            (checkout / "Cargo.toml").write_text('[package]\nname = "fixture"\nversion = "0.1.0"\n')
            (checkout / ".cargo-ok").write_text("ok\n")
            subprocess.run(["git", "-C", str(checkout), "add", "Cargo.toml"], check=True)
            subprocess.run(
                ["git", "-C", str(checkout), "-c", "user.name=Vendor Fixture",
                 "-c", "user.email=vendor@example.invalid", "commit", "-qm", "fixture"],
                check=True,
            )
            head = subprocess.check_output(
                ["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True
            ).strip()
            (checkout / ".git/HEAD").write_text(f"{head}\n")
            subprocess.run(
                ["git", "-C", str(checkout), "update-ref", "-d", "refs/heads/main"],
                check=True,
            )
            self.assertTrue((checkout / ".git/refs").is_dir())
            self.assertFalse(any(path.is_file() for path in (checkout / ".git/refs").rglob("*")))

            relative = f"git-cache/checkouts/fixture/{head[:7]}"
            files = {}
            bundle.add_tree(files, checkout, relative, allow_symlinks=True)
            archive_path = root / "source.tar.gz"
            bundle.export(files, archive_path)
            extracted = root / "extracted"
            extracted.mkdir()
            with tarfile.open(archive_path, "r:gz") as archive:
                refs = next(member for member in archive.getmembers()
                            if member.name.rstrip("/") == f"{relative}/.git/refs")
                self.assertTrue(refs.isdir())
                self.assertEqual(refs.mode, 0o755)
                archive.extractall(extracted, filter="data")
            extracted_checkout = extracted / relative
            self.assertTrue((extracted_checkout / ".git/refs").is_dir())
            observed = subprocess.run(
                ["git", "-C", str(extracted_checkout), "rev-parse", "HEAD"],
                capture_output=True, text=True,
            )
            self.assertEqual(observed.returncode, 0, observed.stderr)
            self.assertEqual(observed.stdout.strip(), head)
            object_type = subprocess.run(
                ["git", "-C", str(extracted_checkout), "cat-file", "-t", head],
                capture_output=True, text=True,
            )
            self.assertEqual(object_type.returncode, 0, object_type.stderr)
            self.assertEqual(object_type.stdout.strip(), "commit")

            (extracted_checkout / ".git/HEAD").write_text("0" * 40 + "\n")
            tampered = subprocess.run(
                ["git", "-C", str(extracted_checkout), "rev-parse", "HEAD"],
                capture_output=True, text=True,
            )
            self.assertFalse(tampered.returncode == 0 and tampered.stdout.strip() == head)

    def test_export_rejects_escaping_symlink_and_noncanonical_directory(self):
        with tempfile.TemporaryDirectory(prefix="vendor-bundle-unsafe-") as temporary:
            root = Path(temporary)
            checkout = root / "checkout"
            checkout.mkdir()
            outside = root / "outside"
            outside.mkdir()
            (checkout / "escape").symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "symlink escapes declared Git checkout"):
                bundle.add_tree({}, checkout, "git-cache/checkouts/fixture/revision", allow_symlinks=True)

            archive_path = root / "unsafe.tar.gz"
            with self.assertRaisesRegex(ValueError, "unsafe or noncanonical archive path"):
                bundle.export({"git-cache/../escape": outside}, archive_path)
            self.assertFalse(archive_path.exists())


if __name__ == "__main__":
    unittest.main()
