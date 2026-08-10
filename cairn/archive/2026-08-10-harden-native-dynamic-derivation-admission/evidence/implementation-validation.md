# Implementation validation

Date: 2026-08-10

Implementation commit: `015813626bc6f2f875dc0c2682d5129cf9c3c150`

## Result

The native dynamic admission path now rejects incomplete identity facts before registry or scheduler mutation.
The all-zero parent fallback is removed from production code.

The private state sequence is parsed, validated, identity-resolved, and registry-ready.
The Worker observes parent and registry facts between pure stages.
Only the final state can enter `DerivationRegistry::insert_registry_ready_dynamic`.

## Implemented requirements

- Traditional covered derivations keep their prior HDM and configured-prefix path behavior.
- `DrvWithVersion("xp-dyn-drv",...)` has one bounded iterative parser.
- Unknown versions, unsupported outputs, malformed trees, empty requests, and excess depth fail closed.
- Parent facts reject missing, duplicate, conflicting, unexpected, wrong-prefix, and wrong-domain inputs.
- The native hash domain remains BLAKE3.
- The Nix compatibility dependency remains outside the native core.
- Exact duplicates are idempotent.
- Batch collisions fail before the first insertion.
- Missing-parent and existing-path collision tests prove unchanged Worker and registry state.

## Former fallback characterization

The baseline fallback produced this HDM:

```text
3e0c7a40bf762ac25cd6197d36fe18f3b322a0d0d972c48a1fdad53565f3f906
```

It produced this path:

```text
/nix/store/rg33vjgs3ksf3d0g5c2gjqcf0np64dxz-hello.drv
```

The new path returns `dynamic-admission-missing-parent` instead.
See `zero-fallback-characterization.log`.

## Focused validation

The exact command transcript is `implementation-validation.log`.

```text
nix develop -c rustfmt --edition 2024 --check \
  crates/crunch-build/src/dynamic.rs \
  crates/crunch-build/src/lib.rs \
  crates/crunch-build/src/registry.rs \
  crates/crunch-build/src/worker.rs

nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-dynamic-admission-target \
  cargo test -p crunch-build --lib --tests

nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-dynamic-admission-target \
  cargo clippy -p crunch-build --lib --tests --no-deps -- -D warnings

nix develop -c cargo -Zscript scripts/check-dynamic-admission-boundary.rs --self-test
nix develop -c cargo -Zscript scripts/check-dynamic-admission-boundary.rs
nix develop -c cargo -Zscript scripts/check-nix-derivation-boundary.rs
git diff --check
```

Results:

- `crunch-build`: 674 unit tests passed.
- `export_api`: one integration test passed.
- focused first-party Clippy passed with warnings denied.
- targeted rustfmt passed.
- positive and negative boundary self-tests passed.
- the dynamic source boundary passed.
- the `nix-derivation` source guard passed.
- `git diff --check` passed.

## Broad validation findings

`cargo test --workspace --lib --tests --no-fail-fast` completed all targets.
The changed `crunch-build` package passed all 674 tests.
Thirteen unrelated root and integration targets failed.
The failures cover bootstrap parity, seccomp, stale catalogs, foreign-import CLI expectations, and host-dependent integration paths.
See `workspace-tests.log` for exact output.

The checked-in first-party Clippy gate stopped in `mantlepkgs-core`.
It reported two unrelated `absurd_extreme_comparisons` findings in `versions.rs`.
The focused changed-package Clippy command passed.
See `first-party-clippy.log`.

The focused Tiger Style command found no issue in `dynamic.rs` or the changed Worker path.
It stopped on existing `build_request.rs`, `execution_profile.rs`, `registry.rs`, and `worker.rs` findings.
See `focused-tigerstyle.log`.

Repo-wide rustfmt found only the known unrelated drift in `src/source_built_fixed_point_shell.rs`.
The four changed Rust files passed targeted rustfmt.
See `fmt-all.log`.

`nix build .#crunch -L` compiled the release output.
Its test phase reached 173 passes before the known seccomp listener test failure.
See `nix-package-check.log`.

## Artifact identity

`admission-receipt.ncl` binds the implementation commit and selected artifacts with BLAKE3.
It also records the policy limits, compatibility forms, former fallback, validation, and bounded findings.

## Rollback

Rollback must restore `dynamic.rs`, `registry.rs`, and the Worker adapter together.
The rollback revision restores the unsupported all-zero parent fallback.
Operators must record that risk if they select the rollback.

## Claim boundary

The evidence proves only the recorded parsing, validation, identity, path, and registry-plan behavior.
It does not prove builder correctness, source trust, sandboxing, output correctness, scheduler quality, or release eligibility.
