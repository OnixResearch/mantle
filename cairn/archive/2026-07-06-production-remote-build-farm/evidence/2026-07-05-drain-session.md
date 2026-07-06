# Drain Session Evidence — 2026-07-05

## Summary

Drained the cache-substitution implementation (committed), assessed both active changes, and updated tasks to reflect true implementation state.

## Commit

```
commit aa7d70d0
Implement ordered cache substitution with advisory metadata, castore
completeness, and structured admission diagnostics

19 files changed, 1242 insertions(+), 81 deletions(-)
create mode 100644 crates/crunch-store/src/completeness.rs
create mode 100644 crates/crunch-store/src/metadata_cache.rs
```

## Test Results

All 88 remote_build tests pass (cached):
```
test result: ok. 88 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out
```

All 39 cache_substitution tests pass:
```
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 1209 filtered out
```

All 8 completeness tests pass:
```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 157 filtered out
```

All 8 metadata_cache tests pass:
```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 157 filtered out
```

## Cairn Gates

Both changes pass all gates and validation (18 specs, 2 changes, 0 issues).

## Task Updates

### production-remote-build-farm
Marked done (existing infrastructure):
- I2 — cryptographic output trust (validate_remote_builder_frames_output_import, key material verification, tests)
- V2 — negative route validation (coordinator tests for prefix/capability/untrusted rejection)
- V3 — positive cryptographic trust (output trust tests)
- V4 — negative cryptographic trust (untrusted key, wrong material, unsigned pathinfo)
- V5 — positive streaming transfer (NAR/frame transfer tests)
- V6 — negative transfer (tampered payload, missing artifact tests)
- V7 — positive coordinator (dispatch/registration/redelivery tests)
- V8 — negative coordinator (capability/concurrency/conflict/log bounding tests)
- V13 — gates + focused tests (88 passing, all 3 gates pass)

Remaining (need new code):
- I1 — scheduler integration
- I3 — chunked CAS streaming, resume cursors
- I4 — durable coordinator persistence
- I5, I6, I7 — publication, config, CI boundary
- V1 — multi-root scheduler test
- V9, V10 — publication tests
- V11 — config tests
- V12 — CI boundary tests

### cache-substitution-reuse-diagnostics
Implementation fully committed. Verification V1-V6 remain unchecked — need integration-level tests at the store/Pipeline layer.