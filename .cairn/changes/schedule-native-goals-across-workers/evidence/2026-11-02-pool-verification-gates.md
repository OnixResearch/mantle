# Verification gates for the pool change (2026-11-02)

Scope: S13's gate runs on the current tree. S13 is not claimed complete.

## Passed

```text
cargo test -p crunch-remote-core -p crunch-remote-app -- --test-threads 1
  -> ok: 10, 119, 1, 0, 1 tests across the crate targets
cargo test -p mantle --test remote_transfer_production -- --test-threads 1
  -> ok. 34 passed
cargo test -p crunch-build --lib -- --test-threads 1
  -> ok. 789 passed
cargo test -p mantle --test example_projects signed_cache_roundtrip_substitutes_through_an_admitted_trusted_cache
  -> ok. 1 passed
strict clippy (crunch-build, crunch-pipeline, mantle --tests, -D warnings) -> exit 0
cargo fmt --check (crunch-build, crunch-pipeline, mantle) -> exit 0
cairn validate -> issues [], change_issues [], receipt recorded
```

## Blocked

```text
bash scripts/check-no-std-core.sh
  -> "rustup not found and target wasm32-unknown-unknown is not preinstalled
     under the active rustc sysroot"
```

The host toolchain has no `wasm32-unknown-unknown` target and no rustup to add
it, so the no-std boundary check cannot run here. The ordinary quality-gate
script includes the same step and is therefore not run.

## Non-claims

- S13 is not complete; the no-std boundary and the ordinary gate remain
  unevidenced on this host.
- S14 stays ordered behind S13 and is unchecked.
