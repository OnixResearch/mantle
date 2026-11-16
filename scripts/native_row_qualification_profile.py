"""Bounded streaming selection of the Mantle source record from a verified SourceBundleManifest.

The profile can contain many gigabytes of inline source payloads. Mantle's own bundle
verify action validates the complete manifest; this reader only extracts the one
classified source record's path/digest index without materializing unrelated payloads.
"""
# r[impl bootstrap_inventory.cold_stagex_native_row_adoption]
import json
import stat
from pathlib import Path

MAX_BUNDLE_BYTES = 4_398_046_511_104  # mantle_application_contract::MAX_SOURCE_BUNDLE_EFFECT_BYTES


class BundleReader:
    def __init__(self, path):
        self.path = Path(path)
        self.file = self.path.open("rb")
        self.lookup = self.path.open("rb")
        self.buffer = b""
        self.cursor = 0
        self.position = 0

    def peek(self):
        if self.cursor == len(self.buffer):
            self.buffer = self.file.read(1024 * 1024)
            self.cursor = 0
        if not self.buffer:
            raise ValueError("truncated source bundle")
        return self.buffer[self.cursor]

    def take(self):
        value = self.peek()
        self.cursor += 1
        self.position += 1
        return value

    def whitespace(self):
        while self.peek() in b" \r\n\t":
            self.take()

    def expect(self, expected):
        self.whitespace()
        if self.take() != ord(expected):
            raise ValueError(f"invalid source bundle structure: expected {expected}")

    def string(self, capture=False):
        self.expect('"')
        parts = bytearray() if capture else None
        while True:
            self.peek()
            start = self.cursor
            quote = self.buffer.find(b'"', start)
            slash = self.buffer.find(b"\\", start)
            end = min((x for x in (quote, slash) if x >= 0), default=len(self.buffer))
            if parts is not None:
                parts.extend(self.buffer[start:end])
                if len(parts) > 4096:
                    raise ValueError("source bundle key/path exceeds operator bound")
            advanced = end - start
            self.cursor += advanced
            self.position += advanced
            if end == len(self.buffer):
                continue
            symbol = self.take()
            if symbol == ord('"'):
                break
            if parts is not None:
                parts.append(symbol)
            escaped = self.take()
            if parts is not None:
                parts.append(escaped)
        if capture:
            return json.loads(b'"' + parts + b'"')

    def value(self, depth=0):
        if depth > 64:
            raise ValueError("source bundle nesting exceeds bound")
        self.whitespace()
        opening = self.peek()
        if opening == ord('"'):
            self.string()
        elif opening in (ord('{'), ord('[')):
            closing = ord('}') if self.take() == ord('{') else ord(']')
            self.whitespace()
            if self.peek() == closing:
                self.take()
                return
            while True:
                if opening == ord('{'):
                    self.string()
                    self.expect(':')
                self.value(depth + 1)
                self.whitespace()
                separator = self.take()
                if separator == closing:
                    break
                if separator != ord(','):
                    raise ValueError("invalid source bundle separator")
        else:
            while self.peek() not in b",]} \r\n\t":
                self.take()

    def small(self, limit=16_384):
        start = self.position
        self.value()
        size = self.position - start
        if size > limit:
            raise ValueError("source bundle field exceeds operator bound")
        self.lookup.seek(start)
        return json.loads(self.lookup.read(size))

    def entries(self, read_entry):
        self.expect('[')
        self.whitespace()
        if self.peek() == ord(']'):
            self.take()
            return
        while True:
            yield read_entry()
            self.whitespace()
            separator = self.take()
            if separator == ord(']'):
                break
            if separator != ord(','):
                raise ValueError("invalid source bundle array separator")

    def fields(self, read_field):
        self.expect('{')
        self.whitespace()
        if self.peek() == ord('}'):
            self.take()
            return
        while True:
            key = self.string(capture=True)
            self.expect(':')
            read_field(key)
            self.whitespace()
            separator = self.take()
            if separator == ord('}'):
                break
            if separator != ord(','):
                raise ValueError("invalid source bundle object separator")

    def close(self):
        self.file.close()
        self.lookup.close()


def mantle_source_paths(path, expected_manifest, required_paths):
    """Read only indexed NCL entries; Mantle verifies every record and digest first."""
    path = Path(path)
    size = path.stat().st_size
    if not stat.S_ISREG(path.lstat().st_mode) or not 0 < size <= MAX_BUNDLE_BYTES or path.is_symlink():
        raise ValueError("source bundle exceeds admitted read bound or is not a regular file")
    reader = BundleReader(path)
    fields = {}
    matches = []
    required = set(required_paths)

    def record():
        metadata = None
        kind = None
        paths = {}

        def file_entry():
            values = {}
            def field(key):
                if key in ("path", "file_type", "blake3", "chunk_index", "chunk_count"):
                    values[key] = reader.small(4096)
                else:
                    reader.value()
            reader.fields(field)
            relative = values.get("path")
            if relative in required:
                if relative in paths:
                    raise ValueError(f"chunked or duplicate row source in bundle: {relative}")
                if values.get("file_type") != "regular" or values.get("chunk_index") not in (None, 0) or values.get("chunk_count") not in (None, 1):
                    raise ValueError(f"row source is not a regular unchunked file: {relative}")
                paths[relative] = values.get("blake3")

        def field(key):
            nonlocal metadata, kind
            if key == "kind":
                kind = reader.small(256)
            elif key == "metadata":
                metadata = reader.small(16_384)
            elif key == "files" and isinstance(metadata, dict) and metadata.get("bootstrap_profile_class") == "mantle-source":
                for _ in reader.entries(file_entry):
                    pass
            else:
                reader.value()
        reader.fields(field)
        if isinstance(metadata, dict) and metadata.get("bootstrap_profile_class") == "mantle-source":
            if metadata.get("bootstrap_profile_mode") != "source-built-fixed-point" or kind != "local-path":
                raise ValueError("Mantle source record kind or profile mode mismatch")
            matches.append(paths)

    def top(key):
        if key == "records":
            for _ in reader.entries(record):
                pass
        elif key in ("format", "version", "store_prefix", "manifest_blake3"):
            fields[key] = reader.small(4096)
        else:
            reader.value()

    try:
        reader.fields(top)
        while reader.position < size and reader.peek() in b" \r\n\t":
            reader.take()
        if reader.position != size:
            raise ValueError("trailing source bundle data")
    finally:
        reader.close()
    if fields != {"format": "mantle-source-bundle-v1", "version": 1,
                  "store_prefix": "/mantle/store", "manifest_blake3": expected_manifest}:
        raise ValueError("prepared source bundle identity/prefix mismatch")
    if len(matches) != 1 or set(matches[0]) != required:
        raise ValueError("prepared source bundle lacks exact Mantle row source entries")
    return matches[0]
