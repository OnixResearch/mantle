# Content-addressed pool goal slice (2026-11-01)

Scope: let a content-addressed (CA) derivation, the default addressing mode,
cross the pool goal payload and admit its realized output.

## What changed

- `src/remote_build.rs`: `parse_remote_derivation_json` accepts the canonical
  derivation JSON first, then rebuilds from the wire shape. This is required
  because `nix_compat::derivation::Output` serializes an unset output path (a
  CA output) as `"path": null` while its derived deserializer accepts only a
  string. Used by the pool executor, the receiver closure validation, the local
  executor registry, and workspace-policy inspection.
- `src/remote_build/pool_goal_executor.rs`: when the request's output paths are
  not the derivation's declared paths (CA placeholders), match the admitted
  outputs by result order instead of by logical-path suffix.
- `crates/crunch-build/src/distributed.rs`,
  `crates/crunch-build/src/orchestrate.rs`, `src/remote_build.rs`: closure
  entries now carry resolved output paths for outputs the derivation does not
  declare. The Worker resolves them from its registry, the receiver registers
  them for CA dependencies, and the pool executor uses them to build a
  dependent goal's input refs. Entries are bounded and validated; a new unit
  test rejects duplicate or malformed output names.
- `crates/crunch-build/src/orchestrate.rs`: the pool closure root uses the
  dispatched goal's prepared derivation, and non-root entries use the resolved
  derivation when CA input resolution rewrote the action.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 16 passed; 0 failed; 0 ignored` (36.22s) |
| `cargo test -p crunch-build --lib -- distributed worker -- --test-threads 1` | `test result: ok. 224 passed; 0 failed; 0 ignored` |
| `cargo test -p mantle --bin mantle -- content_addressed_derivation_json_round_trips` | `test result: ok. 1 passed` |
| `cargo clippy -p crunch-build -p crunch-remote-core -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle -p crunch-build -p crunch-remote-core` | exit 0 |

The suite now includes `production_pool_dispatches_one_content_addressed_goal_through_the_worker`
alongside the input-addressed single goal and two-worker diamond.

## CA dependency resolution

A CA-resolved action has two identities: the scheduler identity (the evaluated
`.drv` path, used for dedupe and waiters) and the action identity (the resolved
derivation and its own path). The prepared pool goal now carries the resolved
path (`PreparedBuild::resolved_drv_path`) and the closure root is keyed by the
bytes actually dispatched. CA inputs realized by resolution move from
`inputDerivations` to `inputSources`, so the remaining dependency entries stay
keyed by their unresolved original paths.

`production_pool_dispatches_a_content_addressed_diamond_across_two_workers`
now passes: the shared CA dependency is built once, both branches resolve its
realized path, and the joined output bytes match.

Non-claims: same-host stdio loopback only; one worker generation; no
cancellation, stale fence, worker death, dynamic-plan return, or resource
saturation case.
