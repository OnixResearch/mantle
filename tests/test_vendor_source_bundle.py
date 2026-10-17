"""Behavioral regressions for the bounded lock-vendor source archive."""

import hashlib
import importlib.util
import os
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
REPLAY_SCRIPT = (
    Path(__file__).resolve().parents[1]
    / "scripts/lock-vendor-replay/replay.py"
)
replay_spec = importlib.util.spec_from_file_location("lock_vendor_replay", REPLAY_SCRIPT)
assert replay_spec is not None and replay_spec.loader is not None
replay = importlib.util.module_from_spec(replay_spec)
replay_spec.loader.exec_module(replay)


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

    def test_cargo_checkout_requires_exact_local_object_database_and_locked_revision(self):
        with tempfile.TemporaryDirectory(prefix="vendor-bundle-cache-origin-") as temporary:
            root = Path(temporary)
            source = root / "source"
            db = root / "cache/db/fixture"
            subprocess.run(["git", "init", "-q", "--initial-branch=main", str(source)], check=True)
            (source / "Cargo.toml").write_text('[package]\nname = "fixture"\nversion = "0.1.0"\n')
            subprocess.run(["git", "-C", str(source), "add", "Cargo.toml"], check=True)
            subprocess.run(
                ["git", "-C", str(source), "-c", "user.name=Vendor Fixture",
                 "-c", "user.email=vendor@example.invalid", "commit", "-qm", "fixture"],
                check=True,
            )
            revision = subprocess.check_output(
                ["git", "-C", str(source), "rev-parse", "HEAD"], text=True
            ).strip()
            checkout = root / "cache/checkouts/fixture" / revision[:7]
            db.parent.mkdir(parents=True)
            checkout.parent.mkdir(parents=True)
            subprocess.run(["git", "clone", "-q", "--bare", str(source), str(db)], check=True)
            subprocess.run(["git", "clone", "-q", str(db), str(checkout)], check=True)
            subprocess.run(["git", "-C", str(checkout), "remote", "set-url", "origin", db.resolve().as_uri()], check=True)
            (checkout / ".cargo-ok").write_text("ok\n")
            self.assertEqual(
                subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip(),
                revision,
            )
            git_environment = {
                "PATH": os.environ["PATH"], "HOME": "/nonexistent",
                "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null",
                "GIT_TERMINAL_PROMPT": "0",
            }
            bundle._verify_checkout_revision_and_local_origin(checkout, db, revision, git_environment)
            with self.assertRaisesRegex(ValueError, "revision drift"):
                bundle._verify_checkout_revision_and_local_origin(checkout, db, "0" * 40, git_environment)
            subprocess.run(
                ["git", "-C", str(checkout), "remote", "set-url", "origin", "https://example.invalid/forged.git"],
                check=True,
            )
            with self.assertRaisesRegex(ValueError, "not its declared Cargo object DB"):
                bundle._verify_checkout_revision_and_local_origin(checkout, db, revision, git_environment)

    def test_missing_registry_archive_uses_locked_fixed_fetch_and_present_tamper_denies(self):
        with tempfile.TemporaryDirectory(prefix="vendor-registry-source-") as temporary:
            cache = Path(temporary)
            artifact = b"locked registry archive"
            digest = hashlib.sha256(artifact).hexdigest()
            row = ("fixture", "1.2.3", digest, "flat")
            uri = "https://static.crates.io/crates/fixture/fixture-1.2.3.crate"
            self.assertEqual(replay.registry_source(cache, row), uri)

            entry = cache / "fixture-1.2.3.crate"
            entry.write_bytes(artifact)
            self.assertEqual(replay.registry_source(cache, row), entry.as_uri())
            entry.write_bytes(b"tampered")
            with self.assertRaisesRegex(RuntimeError, "wrong locked archive"):
                replay.registry_source(cache, row)
            entry.unlink()
            entry.symlink_to(cache / "nonexistent.crate")
            with self.assertRaisesRegex(RuntimeError, "wrong locked archive"):
                replay.registry_source(cache, row)

    def test_source_archive_accepts_pinned_checkout_link_but_rejects_escape(self):
        with tempfile.TemporaryDirectory(prefix="vendor-bundle-checkout-link-") as temporary:
            root = Path(temporary)
            checkout = root / "checkout"
            (checkout / "src").mkdir(parents=True)
            (checkout / "evaluations").mkdir()
            (checkout / "src/lib.rs").write_text("pub fn pinned() {}\n")
            (checkout / "evaluations/native-fskit").symlink_to("../src", target_is_directory=True)
            checkout_prefix = "git-cache/checkouts/fixture/abc1234"
            files = {}
            bundle.add_tree(files, checkout, checkout_prefix, allow_symlinks=True)
            source_archive = root / "source.tar.gz"
            bundle.export(files, source_archive)
            observed = replay.unpacked_nar(source_archive, root / "accepted")
            self.assertEqual(len(observed), 64)
            extracted = root / "accepted/source-tree" / checkout_prefix
            self.assertTrue((extracted / "evaluations/native-fskit").is_symlink())
            self.assertEqual((extracted / "evaluations/native-fskit").resolve(), extracted / "src")

            unsafe_archive = root / "unsafe.tar.gz"
            with tarfile.open(unsafe_archive, "w:gz") as archive:
                link = tarfile.TarInfo(f"{checkout_prefix}/evaluations/native-fskit")
                link.type = tarfile.SYMTYPE
                link.linkname = "../../../../../../etc"
                archive.addfile(link)
            with self.assertRaisesRegex(RuntimeError, "Git checkout symlink escapes pinned source"):
                replay.unpacked_nar(unsafe_archive, root / "denied")
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
