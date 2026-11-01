# Pool ambiguous output keys slice (2026-11-01)

Scope: a pool endpoint that declares two trusted output keys with the same name
must fail configuration admission before any dispatch or store effect.

## What changed

- `tests/remote_transfer_production.rs`:
  `production_pool_rejects_ambiguous_output_keys_before_dispatch` writes a pool
  config whose single endpoint has two `cache.example.com-1` keys with different
  material, runs a pool build, and asserts the configuration is rejected
  (`trusted output keys must be unique by name and by material`) with an empty
  client store.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 27 passed; 0 failed; 0 ignored` (73.27s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

The core `AmbiguousOutputKey` rule remains covered by
`crates/crunch-remote-core/src/pool_tests.rs`; this test proves the CLI pool
path reaches the configuration check before any transport or store mutation.

## Non-claims

- Only the same-name/different-material case is exercised; the loader check also
  rejects duplicate materials, but that variant is not separately run.
- Missing/extra outputs and private inputs are still uncovered.
