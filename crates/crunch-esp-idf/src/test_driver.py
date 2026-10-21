"""Contract tests, not firmware/toolchain evidence. Run with the pinned Python closure."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("driver", Path(__file__).with_name("driver.py"))
DRIVER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DRIVER)


class ConfigurationTests(unittest.TestCase):
    def test_order_and_explicit_unset(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = [root / name for name in ("common", "target", "sdkconfig")]
            files[0].write_text("CONFIG_STACK=4096\nCONFIG_FEATURE=y\n")
            files[1].write_text("CONFIG_STACK=4608\n")
            files[2].write_text("CONFIG_STACK=8192\n# CONFIG_FEATURE is not set\n")
            self.assertEqual(DRIVER.parse_config(files), {"CONFIG_STACK": "8192", "CONFIG_FEATURE": "n"})

    def test_target_and_reproducibility_cannot_be_overridden(self):
        for values in ({"CONFIG_IDF_TARGET": '"esp32c3"'}, {"CONFIG_APP_REPRODUCIBLE_BUILD": "n"},
                       {"CONFIG_SECURE_FLASH_ENC_ENABLED": "y"}):
            with self.assertRaises(RuntimeError):
                DRIVER.mandatory_config(values, "esp32s3", "partitions.csv")
        policy = {}
        DRIVER.mandatory_config(policy, "esp32s3", "partitions.csv")
        self.assertEqual(policy["CONFIG_APP_REPRODUCIBLE_BUILD"], "y")
        self.assertEqual(policy["CONFIG_IDF_TARGET"], '"esp32s3"')


class InventoryTests(unittest.TestCase):
    def test_content_modes_and_layout_bind_identity_but_absolute_directory_does_not(self):
        with tempfile.TemporaryDirectory() as directory:
            roots = [Path(directory) / name for name in ("first", "second")]
            for root in roots:
                root.mkdir()
                (root / "input").write_bytes(b"firmware-source")
            before = DRIVER.inventory(roots[0], roots)
            self.assertEqual(before, DRIVER.inventory(roots[1], roots))
            (roots[1] / "input").write_bytes(b"changed-source")
            self.assertNotEqual(before, DRIVER.inventory(roots[1], roots))
            (roots[1] / "input").write_bytes(b"firmware-source")
            (roots[1] / "input").chmod(0o755)
            self.assertNotEqual(before, DRIVER.inventory(roots[1], roots))
            (roots[1] / "input").chmod(0o644)
            (roots[1] / "input").rename(roots[1] / "renamed")
            self.assertNotEqual(before, DRIVER.inventory(roots[1], roots))

    def test_symlink_escape_is_not_a_declared_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "input"
            root.mkdir()
            secret = Path(directory) / "undeclared"
            secret.write_bytes(b"outside")
            (root / "escape").symlink_to(secret)
            with self.assertRaisesRegex(RuntimeError, "escapes declared closure"):
                DRIVER.inventory(root, [root])


if __name__ == "__main__":
    unittest.main()
