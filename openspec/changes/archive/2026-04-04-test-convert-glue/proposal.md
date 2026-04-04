## Why

`convert.rs` is the most load-bearing code in crunch-glue — it computes
BLAKE3 store paths, wires input_derivations, propagates FOD ca_hash,
detects cycles, and deduplicates diamond dependencies. It has zero direct
unit tests. The 13 tests in `tests.rs` cover serde deserialization, and
integration tests exercise `convert()` indirectly, but no test targets the
conversion logic in isolation. Bugs in store path computation silently
produce wrong paths with no assertion catching them.

## What Changes

- Add unit tests to crunch-glue covering `convert()` and `convert_inner()`.
- Test `known_paths.rs` methods directly (insert, lookup, cycle detection).

## Capabilities

### New Capabilities
- `test-convert-store-paths`: Verify output path computation for simple,
  multi-output, and fixed-output derivations against known-good values.
- `test-convert-inputs`: Verify input_derivations and input_sources wiring
  for nested, diamond, and source-only dependency graphs.
- `test-convert-cycle`: Verify cycle detection rejects circular deps.
- `test-convert-dedup`: Verify diamond dependencies produce a single entry
  in KnownPaths.
- `test-known-paths-api`: Verify KnownPaths insert/get/hdm lookups.

## Impact

- **Files**: `crates/crunch-glue/src/convert.rs` (test module added),
  `crates/crunch-glue/src/known_paths.rs` (test module added)
- **APIs**: None changed
- **Dependencies**: None
- **Testing**: All new tests run with `cargo test -p crunch-glue`
