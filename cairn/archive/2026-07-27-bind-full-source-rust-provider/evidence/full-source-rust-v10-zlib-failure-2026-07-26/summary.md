# Full-source Rust v10 zlib failure

## Result

The mrustc-to-Rust 1.90.0 first stage completed.
Its retained build manifest and provider receipt bind these current identities:

- parallel jobs: `4`
- native provider BLAKE3: `f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f`
- source-closure BLAKE3: `7e93ccc7a29bacc1da6f75c10ae90655ee83afd0d95ca282c8d270c225372f45`
- stage-plan BLAKE3: `df165d6eab6789db24c7a4a7106b441b44e544b53c2a77daa4fa468fb26f2d8c`
- stage-script BLAKE3: `a74dd91f34ae2fadedd32a982338a5aa3e25d9e09a58fc6401cb37c6dffb8e37`

The Rust 1.91.1 stage failed before provider publication.

## Exact failure

LLVM found CMake's receipt-bound zlib archive and headers.
That zlib uses prefixed API macros such as `crc32 -> cm_zlib_crc32` and `compress -> cm_zlib_compress`.
The macros rewrote LLVM function declarations and definitions.
Compilation then failed in `CRC.cpp` and `Compression.cpp`.

The exact diagnostics are in `rustc-stage1-build.log.zst`. Decompress the file before you inspect lines 346-385.
The detached command returned status `1` and published no final provider.

## Repair boundary

Rust's bootstrap config supports `[llvm].build-config` as a map of CMake definitions.
The next attempt must set `LLVM_ENABLE_ZLIB = "OFF"` for every x.py Rust stage.
The mrustc first stage already disables LLVM zlib in its authenticated Makefile normalization.

This repair removes an optional LLVM compression dependency.
It does not add a host fallback, imported runtime, or unverified source.
