#!/usr/bin/env python3
"""Consumer-visible independent reader/numeric boundary tests; stdlib only."""
import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest

MODULE = Path(__file__).resolve().parents[3] / "crates/crunch-model-conversion/src/gguf_reader.py"
spec = importlib.util.spec_from_file_location("mantle_gguf_reader", MODULE)
reader = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reader)


def string(value):
    raw = value.encode()
    return struct.pack("<Q", len(raw)) + raw


def artifact(kind=0, payload=None, duplicate=False):
    metadata = string("general.architecture") + struct.pack("<I", 8) + string("llama")
    tensor = string("test.weight") + struct.pack("<IQIQ", 1, 32, kind, 0)
    count = 2 if duplicate else 1
    header = b"GGUF" + struct.pack("<IQQ", 3, count, 1) + metadata + tensor * count
    header += b"\0" * (-len(header) % 32)
    if payload is None:
        payload = struct.pack("<32f", *([1 / 128] * 32)) if kind == 0 else struct.pack("<e32b", 1 / 128, *([1] * 32))
    return header + payload


class ReaderTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)

    def open(self, name, data):
        path = self.root / name
        path.write_bytes(data)
        return reader.Reader(path)

    def test_f32_q8_values_and_bounded_projection(self):
        f32 = self.open("f32", artifact())
        q8 = self.open("q8", artifact(8))
        self.assertEqual(f32.metadata["general.architecture"], "llama")
        self.assertEqual(q8.tensors["test.weight"]["shape"], [32])
        self.assertEqual(q8.values("test.weight"), [1 / 128] * 32)
        result = reader.compare(f32, q8)
        self.assertEqual(result["max_abs_tensor_error"], 0.0)
        self.assertEqual(result["max_abs_projection_error"], 0.0)
        self.assertTrue(result["scope"]["bounded_tensor_projection_smoke"])
        self.assertFalse(result["scope"]["model_quality"])
        self.assertFalse(result["scope"]["semantic_equivalence"])

    def test_numeric_smoke_rejects_large_error_and_nonfinite(self):
        f32 = self.open("f32", artifact())
        bad = self.open("bad", artifact(8, struct.pack("<e32b", 1.0, *([1] * 32))))
        with self.assertRaisesRegex(ValueError, "numerical smoke bounds"):
            reader.compare(f32, bad)
        nan = self.open("nan", artifact(8, struct.pack("<e32b", float("nan"), *([1] * 32))))
        with self.assertRaisesRegex(ValueError, "nonfinite"):
            nan.values("test.weight")

    def test_corrupt_extent_duplicate_tensor_and_version_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "extent"):
            self.open("truncated", artifact()[:-1])
        with self.assertRaisesRegex(ValueError, "dimensions/name"):
            self.open("duplicate", artifact(duplicate=True))
        broken = bytearray(artifact())
        struct.pack_into("<I", broken, 4, 2)
        with self.assertRaisesRegex(ValueError, "GGUF v3"):
            self.open("version", broken)


if __name__ == "__main__":
    unittest.main()
