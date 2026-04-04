## Why

`verify_fod_hash` handles `CAHash::Flat` for all four hash algorithms and
`CAHash::Nar(Sha256)`, but `CAHash::Nar` with md5, sha1, or sha512 logs a
warning and skips verification. A FOD declaring `mode = "recursive"` with
`algo = "sha1"` would pass even if the content is wrong.

This matters less in practice (sha256 is near-universal for NAR hashing)
but it's a correctness gap — the code accepts a hash contract it doesn't
enforce.

## What Changes

- Compute NAR serialization and hash it with the declared algorithm when
  `CAHash::Nar` uses a non-sha256 hash.
- Reuse the existing `SimpleRenderer` / `write_nar` machinery, piping NAR
  bytes through the appropriate hasher instead of only sha256.

## Capabilities

### Modified Capabilities
- `fod-verification`: Extend to verify NAR-mode FODs for all four hash
  algorithms (md5, sha1, sha256, sha512).

## Impact

- **Files**: `crates/crunch-build/src/orchestrate.rs`
- **APIs**: `verify_fod_hash` internal signature may change
- **Dependencies**: May need `write_nar` exposed from snix-store or a
  custom NAR writer wrapper
- **Testing**: New tests for NAR md5/sha1/sha512 match and mismatch
