# Immutable release pointer verification

Date: 2026-08-08

## Lifecycle tool boundary

Mantle still stores lifecycle data under `cairn/`. Cairn revision `0136f976ff4f093be22aa2782227709a5873931b` reads only `.cairn/` and reports zero Mantle changes.

This verification used Cairn revision `e5ee2a61d8561d8fb47f42012b5d23211f847e7e`. This is the last local revision that reads Mantle's current `cairn/` layout.

The following lifecycle commands passed:

```text
nix run path:<cairn-e5ee2a6>#cairn -- validate --root <worktree>
"valid": true

nix run path:<cairn-e5ee2a6>#cairn -- gate proposal immutable-release-current-pointer --root <worktree>
"valid": true
"verdict": "PASS"

nix run path:<cairn-e5ee2a6>#cairn -- gate design immutable-release-current-pointer --root <worktree>
"valid": true
"verdict": "PASS"

nix run path:<cairn-e5ee2a6>#cairn -- gate tasks immutable-release-current-pointer --root <worktree>
"valid": true
"verdict": "PASS"
```

Tracey passed:

```text
traceability coverage ok: 155/155 referenced (profile mantle-default)
```

## Focused package and shell tests

```text
nix develop -c cargo test -p crunch-release-core --lib
test result: ok. 232 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

nix develop -c cargo test -p mantle --bin mantle release_current_pointer::
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2296 filtered out; finished in 0.00s
```

These tests include positive and negative cases for immutable objects, pointer changes, rollback, missing objects, symlinks, and evidence replacement.

## Scoped Clippy

```text
nix develop -c cargo clippy -p crunch-release-core --all-targets --no-deps -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.57s

nix develop -c cargo clippy -p mantle --bin mantle --no-deps -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 46.31s
```

## Broad-check results

The required broad commands ran. They found unrelated blockers on the unchanged `origin/main` baseline:

- `./scripts/check-first-party-quality.sh` stopped at formatting in `src/source_built_fixed_point_shell.rs`.
- First-party Clippy stopped at `src/rust_plan.rs:25547` with `clippy::bool-comparison`.
- Workspace tests passed the release tests, then failed three bootstrap-parity digest assertions. The final result was `2232 passed; 3 failed; 68 ignored`.
- The Nix package check omitted two `fixtures/content-bound-requirements/` files from its filtered source.
- `nix flake check path:$PWD -L` stopped at `bootstrap-blocker-inventory` with 115 full-bootstrap findings.

These blockers do not change immutable release pointer behavior. This change does not repair fixed-point, full-source, Rust-plan, or broad Nix source-filter work.

## Non-claims

This evidence does not prove distribution, deployment, release readiness, durable retention, deletion, installer parity, or implementation equivalence with Celld.
