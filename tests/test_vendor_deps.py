"""Behavioral regression for checkout-local vendored source comparisons."""

import importlib.util
import os
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/vendor-deps.py"
spec = importlib.util.spec_from_file_location("vendor_deps", SCRIPT)
assert spec is not None and spec.loader is not None
vendor_deps = importlib.util.module_from_spec(spec)
spec.loader.exec_module(vendor_deps)


class VendorTreeComparisonTest(unittest.TestCase):
    def test_same_size_and_mtime_hand_edit_is_rejected_after_equal_comparison(self):
        with tempfile.TemporaryDirectory(prefix="mantle-vendor-compare-") as temporary:
            root = Path(temporary)
            generated = root / "generated"
            installed = root / "installed"
            generated_nar = generated / "casita/src/nar.rs"
            installed_nar = installed / "casita/src/nar.rs"
            original = b"// " + b"x" * (64 * 1024) + b"\nconst KEY: u8 = 1;\n"
            edited = original.replace(b"KEY: u8 = 1;", b"KEY: u8 = 2;")
            self.assertEqual(len(edited), len(original))
            for path in (generated_nar, installed_nar):
                path.parent.mkdir(parents=True)
                path.write_bytes(original)

            self.assertEqual(vendor_deps.compare_trees(generated, installed), 3)
            original_stat = installed_nar.stat()
            installed_nar.write_bytes(edited)
            os.utime(installed_nar, ns=(original_stat.st_atime_ns, original_stat.st_mtime_ns))
            edited_stat = installed_nar.stat()
            self.assertEqual(edited_stat.st_size, original_stat.st_size)
            self.assertEqual(edited_stat.st_mtime_ns, original_stat.st_mtime_ns)
            self.assertNotEqual(installed_nar.read_bytes(), generated_nar.read_bytes())
            with self.assertRaisesRegex(RuntimeError, "vendor-deps content drift: casita/src/nar.rs"):
                vendor_deps.compare_trees(generated, installed)

            generated_nar.write_bytes(edited)
            self.assertEqual(vendor_deps.compare_trees(generated, installed), 3)


if __name__ == "__main__":
    unittest.main()
