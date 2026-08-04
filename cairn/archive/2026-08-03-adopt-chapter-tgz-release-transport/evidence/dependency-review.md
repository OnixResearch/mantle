# chapter-tgz dependency review

Date: 2026-08-03

## Selected source

- crate: `chapter-tgz`
- version: `0.1.0`
- Cargo requirement: `=0.1.0`
- crates.io checksum: `71d3b546f2d916ddf609e8f8e271c93c9dcd66d7500a87ee8d81d4c159839d7c`
- upstream release commit: `0090e5d002188228a6c94742f99d0c1983e52f6a`
- license: `MIT OR Apache-2.0`
- minimum Rust version: `1.91`
- direct dependencies: `flate2` and `tar`

`Cargo.lock` contains the registry checksum. The root manifest uses the exact version.

## Reviewed surfaces

The review covered the crate documentation, encoder, decoder, reader, writer, `IndependentRead` trait, examples, CI, repository history, and crates.io metadata.

The release was young at review time. Crates.io reported 16 downloads. The upstream repository had no dedicated test files. Mantle therefore keeps format, corruption, bounds, compatibility, and extraction tests in this repository.

## Important behavior

- Output is one standard-compatible gzip and tar stream.
- Opening a chaptered archive walks the marker chain in chapter count time.
- `jump_to_chapter` panics for an invalid index. Mantle validates counts before every indexed loop.
- `IndependentRead` is implemented upstream for `Cursor`, not `File`.
- Cloned `File` handles share an offset. A safe parallel adapter must use `FileExt::read_at` with a private logical offset.
- Chapter reads do not establish complete final gzip trailer validation.
- An ordinary tgz opens as one fallback chapter but has no Mantle transport index.

## Mantle controls

Mantle does not use upstream extraction.

Mantle checks a detached BLAKE3 over all compressed bytes, validates the complete bounded gzip stream, and counts exact 0.1.0 marker prefixes before opening the chapter reader.

Mantle compares all observed metadata with a canonical bounded index. Unpack writes only through capability-relative no-follow operations and uses atomic no-replace publication.

A test-only `FileExt::read_at` adapter measures parallel behavior. Production remains sequential because the measured gain is below its activation threshold.

## Audit result

The dependency is acceptable for an opt-in transport pilot with the exact pin and Mantle-owned controls. It is not approved as a canonical release format or as a replacement for existing interoperability formats.
