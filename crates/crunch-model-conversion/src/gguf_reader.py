"""Independent, bounded GGUF v3 reader. No gguf/torch/numpy imports.

Supports exactly F32 and Q8_0, the initial adapter's finite fragment.
All dimensions returned in GGML order (fastest varying dimension first).
"""
import math
import struct
from pathlib import Path

MAX_BYTES = 128 << 20
MAX_ITEMS = 65536


class Reader:
    def __init__(self, path):
        path = Path(path)
        if not 24 <= path.stat().st_size <= MAX_BYTES:
            raise ValueError("GGUF size bound")
        self.data = path.read_bytes()
        self.pos = 0
        if self.take(4) != b"GGUF" or self.number("I") != 3:
            raise ValueError("expected little-endian GGUF v3")
        count, metadata_count = self.number("Q"), self.number("Q")
        if not 1 <= count <= 256 or metadata_count > 256:
            raise ValueError("GGUF inventory bound")
        self.metadata = {}
        for _ in range(metadata_count):
            key = self.string().decode("utf-8")
            if key in self.metadata:
                raise ValueError("duplicate metadata")
            self.metadata[key] = self.value(self.number("I"))
        self.tensors = {}
        for _ in range(count):
            name = self.string().decode("utf-8")
            nd = self.number("I")
            if not 1 <= nd <= 4 or name in self.tensors:
                raise ValueError("invalid tensor dimensions/name")
            dims = [self.number("Q") for _ in range(nd)]
            if not all(0 < d <= 4096 for d in dims) or math.prod(dims) > 1 << 24:
                raise ValueError("tensor element bound")
            kind, offset = self.number("I"), self.number("Q")
            n = math.prod(dims)
            if kind == 0:
                size = n * 4
            elif kind == 8 and n % 32 == 0 and dims[0] % 32 == 0:
                size = n // 32 * 34
            else:
                raise ValueError("unsupported GGML encoding")
            self.tensors[name] = {"shape": dims, "type": kind, "offset": offset, "bytes": size}
        alignment = self.metadata.get("general.alignment", 32)
        if alignment != 32:
            raise ValueError("unsupported GGUF alignment")
        self.base = (self.pos + alignment - 1) // alignment * alignment
        ranges = []
        for tensor in self.tensors.values():
            start = self.base + tensor["offset"]
            end = start + tensor["bytes"]
            if start % 32 or end > len(self.data):
                raise ValueError("GGUF tensor extent")
            ranges.append((start, end))
        ranges.sort()
        if any(a[1] > b[0] for a, b in zip(ranges, ranges[1:])):
            raise ValueError("overlapping tensor extents")

    def take(self, n):
        if n < 0 or self.pos + n > len(self.data):
            raise ValueError("truncated GGUF")
        result = self.data[self.pos:self.pos + n]
        self.pos += n
        return result

    def number(self, kind):
        return struct.unpack("<" + kind, self.take(struct.calcsize("<" + kind)))[0]

    def string(self):
        n = self.number("Q")
        if n > 1 << 20:
            raise ValueError("GGUF string bound")
        return self.take(n)

    def value(self, kind, depth=0):
        formats = {0: "B", 1: "b", 2: "H", 3: "h", 4: "I", 5: "i", 6: "f", 7: "?", 10: "Q", 11: "q", 12: "d"}
        if kind in formats:
            value = self.number(formats[kind])
            if isinstance(value, float) and not math.isfinite(value):
                raise ValueError("nonfinite metadata")
            return value
        if kind == 8:
            return self.string().decode("utf-8")
        if kind == 9 and depth == 0:
            subtype, n = self.number("I"), self.number("Q")
            if n > MAX_ITEMS:
                raise ValueError("metadata array bound")
            return [self.value(subtype, depth + 1) for _ in range(n)]
        raise ValueError("unsupported metadata type")

    def values(self, name):
        t = self.tensors[name]
        raw = self.data[self.base + t["offset"]:self.base + t["offset"] + t["bytes"]]
        if t["type"] == 0:
            result = list(struct.unpack("<" + "f" * (len(raw) // 4), raw))
        else:
            result = []
            for i in range(0, len(raw), 34):
                scale = struct.unpack_from("<e", raw, i)[0]
                result.extend(scale * q for q in struct.unpack_from("<32b", raw, i + 2))
        if not all(math.isfinite(x) for x in result):
            raise ValueError("nonfinite tensor data")
        return result


def compare(reference, quantized):
    if reference.tensors.keys() != quantized.tensors.keys():
        raise ValueError("quantization tensor inventory drift")
    maximum, projection = 0.0, 0.0
    for name, t in reference.tensors.items():
        if t["shape"] != quantized.tensors[name]["shape"]:
            raise ValueError("quantization shape drift")
        a, b = reference.values(name), quantized.values(name)
        maximum = max(maximum, max(abs(x - y) for x, y in zip(a, b)))
        width = t["shape"][0]
        for row in range(0, len(a), width):
            # Fixed, bounded dot product, not model inference or model quality.
            delta = abs(sum((a[row + i] - b[row + i]) * ((i % 7 - 3) / 7) for i in range(width)))
            projection = max(projection, delta)
    if maximum > 0.002 or projection > 0.02:
        raise ValueError(f"numerical smoke bounds exceeded: {maximum}, {projection}")
    return {"max_abs_tensor_error": maximum, "max_abs_projection_error": projection,
            "tensor_error_limit": 0.002, "projection_error_limit": 0.02,
            "scope": {"bounded_tensor_projection_smoke": True, "model_quality": False, "semantic_equivalence": False}}
