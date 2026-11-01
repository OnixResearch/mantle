# Single-builder/pool parity slice (2026-11-01)

Scope: the existing single-endpoint remote path and the new pool path must
produce identical output bytes for the same derivation.

## What changed

- `tests/remote_transfer_production.rs`:
  `production_single_builder_and_pool_outputs_match` builds the replay fixture
  through the `--builder` single-endpoint path (default generated worker) and
  through the single-endpoint pool, then compares the physical output bytes.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 25 passed; 0 failed; 0 ignored` (74.12s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- One input-addressed replay derivation is compared; signed-cache-hit and CA
  variants are not part of this comparison.
- Byte equality is not a cross-machine reproducibility claim.
- The single-builder worker uses the CLI's generated local worker arguments, not
  a separately provisioned host.
