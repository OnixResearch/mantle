# Final validation — 2026-06-25

Task-ID: V1,V2
Covers: r[rust_package_planning.provider_fixed_point_release_verifier]

## Focused tests

pueue task 141 recorded focused test counts:

```text
running 42 tests
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 797 filtered out; finished in 0.07s
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 827 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 838 filtered out; finished in 0.00s
```

Coverage summary:

- `cargo_free_self_build::` includes valid provider fixed-point proof validation plus negative cases for stage digest mismatch, missing enforced closure, missing bounded non-claims, explicit closure probe failure, and receipt-bound toolchain checks.
- `release_cmd::` includes optional and required provider fixed-point proof request behavior.
- `release_verify_accepts_provider_fixed_point_proof_flags` covers CLI parsing for `--provider-fixed-point-proof` and `--require-provider-fixed-point-proof`.

## Build, diff, and lifecycle gates

pueue task 151 recorded:

```text
cargo-build: pass
git-diff-check: pass
cairn-validate: valid=true verdict=null
cairn-gate-proposal: valid=true verdict=PASS
cairn-gate-design: valid=true verdict=PASS
cairn-gate-tasks: valid=true verdict=PASS
```

The build emitted existing dead-code warnings in `src/rust_plan.rs`; no new build failure was observed.

## Task completion, sync, and archive checks

pueue task 156 reran checks after marking all tasks complete:

```text
post-task-validate: valid=true verdict=null
post-task-gate-tasks: valid=true verdict=PASS
```

pueue task 160 synced the requirement into `cairn/specs/rust-package-planning/spec.md`, archived the change under `cairn/archive/2026-06-25-provider-fixed-point-release-verifier/`, checked that no `1970-01-01` archive directory was created, and reran validation:

```text
post-archive-validate: valid=true verdict=null
r[rust_package_planning.provider_fixed_point_release_verifier] Mantle MUST let release verification validate a provider-backed Cargo-free fixed-point proof bundle as bounded release-adjacent evidence without upgrading the release reproducibility claim.
```

After removing unrelated rustfmt churn from `src/build_report.rs`, pueue task 172 reran validation and diff check:

```text
post-restore-validate: valid=true verdict=null
```
