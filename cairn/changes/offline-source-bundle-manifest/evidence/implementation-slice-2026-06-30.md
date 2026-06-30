# Implementation slice evidence — Offline source bundle manifest

Date: 2026-06-30

## Implemented in this slice

- Added `src/source_bundle.rs` with a Mantle-owned `mantle-source-bundle-v1` manifest model.
- Added deterministic source record canonicalization for declared local source specs (`kind:identity:path`) across all planned source record kinds.
- Added BLAKE3 content refs, manifest digesting, named limits, safe path/symlink rejection, UTF-8 path checks, duplicate record rejection, source-state import with atomic record writes, pin files, list, and verify/imported-state classification.
- Added `mantle source bundle plan|export|list|import|verify` CLI wiring with JSON and human reports.
- Added `--build-root` planning for `plan|export` so selected `.ncl` roots can be evaluated without building; the pure derivation walk derives metadata-only records for fixed fetcher derivations, git fetcher derivations, and declared store-path inputs.
- Import/list/verify code only reads local bundle/source-state files; it performs no network access.

## Current focused evidence

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-source-plan cargo test -p mantle --bin mantle source_bundle::tests
running 8 tests
...
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 955 filtered out
```

## Remaining gap before archive

This is not yet a complete drain of `offline-source-bundle-manifest`: build-root planning is metadata-only, VCS snapshot materialization, generic non-Cargo package-manager mirror fixtures, Cargo adapter metadata, bootstrap/provider/toolchain source-root fixtures, and offline build preflight integration still need implementation and end-to-end tests.
