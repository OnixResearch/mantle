# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.gzip.1.2.4

- Renamed the active OpenSpec change to match the implemented `gzip 1.2.4` source/derivation and documented the upstream `parts.rst` heading mismatch.
- Added provenance and first-consumer comments beside `gzip_src`.
- Required generated `crc.c` to be nonempty.
- Made object compilation collect and check every declared object before linking.
- Added fail-closed output checks for both `gzip` and `gunzip`.
- Added installed-tool `--help` checks and a compression/decompression roundtrip smoke.
