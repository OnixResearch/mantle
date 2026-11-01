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
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 15 passed; 0 failed; 0 ignored` (52.78s) |
| `cargo test -p crunch-build --lib -- pool_dispatch prepared_goal` | `test result: ok. 7 passed; 0 failed; 0 ignored` |
| `cargo test -p mantle --bin mantle -- content_addressed_derivation_json_round_trips` | `test result: ok. 1 passed` |
| `cargo clippy -p crunch-build -p crunch-remote-core -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle -p crunch-build -p crunch-remote-core` | exit 0 |

The suite now includes `production_pool_dispatches_one_content_addressed_goal_through_the_worker`
alongside the input-addressed single goal and two-worker diamond.

## Remaining gap

A CA **dependency** in a pool graph still fails closed at the receiver closure
path check. A CA-resolved action has a scheduler identity (the evaluated
`.drv` path) and an action identity (the resolved derivation and its own path).
`PreparedPoolGoal` carries the scheduler path as `drv_path` while
`derivation_json` is the resolved derivation, so the closure root key and the
carried bytes disagree. Supporting it requires the S3 obligation: carry the
resolved action path explicitly in the prepared envelope, key the closure root
by it, and keep the scheduler key only for dedupe/waiters. The closure-output
plumbing above is the prerequisite for that step and is not yet exercised by an
end-to-end CA dependency.

Non-claims: same-host stdio loopback only; one worker generation; no
cancellation, stale fence, worker death, dynamic-plan return, or resource
saturation case.
