# Pool/local output parity slice (2026-11-01)

Scope: a pool build and a local build of the same derivation must produce
identical output bytes.

## What changed

- `tests/remote_transfer_production.rs`:
  `production_pool_output_matches_local_build` builds the replay fixture locally
  with `--no-substitute` and through the single-endpoint pool, then compares the
  physical output bytes. The shared single-endpoint helper now returns its
  tempdir with the report so output files can be inspected after the run.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 22 passed; 0 failed; 0 ignored` (67.09s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- The comparison covers one input-addressed fixed-output derivation; the CA
  fixture has its own tests but no local-vs-pool byte comparison.
- The single-builder remote path is not compared in this test; it is covered by
  the existing `production_stdio_*` suites and the ordinary gate.
- Byte equality is not a reproducibility claim across machines or toolchains.
