# Final validation — 2026-06-25

Task-ID: V1,V3
Covers: r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point]

## Focused tests

- pueue task 33: `cargo test -p mantle --bin mantle cargo_free_self_build:: -- --nocapture`
  - `test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 794 filtered out; finished in 0.16s`
- pueue task 38: `cargo test -p mantle --bin mantle native_toolchain_closure:: -- --nocapture`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 822 filtered out; finished in 0.01s`
- pueue task 35: `cargo test -p mantle --bin mantle source_toolchain_closure:: -- --nocapture`
  - `test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 781 filtered out; finished in 0.04s`

These cover positive paths for declared unwind alias materialization, receipt-bound compatibility probe PATH use, source-root native closure materialization, and provider/closure validation. Negative coverage includes explicit-closure probe failure, ambient helper rejection, digest mismatch rejection, missing required roles, and disallowed seed/runtime provenance.

## Formatting, build, and lifecycle validation

pueue task 26 recorded:

- rustfmt: pass
- cargo-build: pass
- git-diff-check: pass
- cairn-validate: valid=true
- cairn-gate-proposal: valid=true verdict=PASS
- cairn-gate-design: valid=true verdict=PASS
- cairn-gate-tasks: valid=true verdict=PASS

After marking V1–V3 complete, pueue tasks 42 and 48 reran lifecycle checks:

- cairn-validate: valid=true
- cairn-gate-tasks: valid=true verdict=PASS

## Sync and post-archive validation

pueue task 49 synced the completed requirement into `cairn/specs/rust-package-planning/spec.md` and reran validation:

```text
post-sync-validate: valid=true verdict=null
r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point] Mantle MUST run provider-backed Cargo-free one-shot and fixed-point proof stages with the explicit receipt-bound source-built Rust/native toolchain closure when that closure is supplied.
```

pueue task 52 archived the change under `cairn/archive/2026-06-25-source-built-rust-provider-fixed-point/`, checked that no `1970-01-01` archive directory was created, and reran validation:

```text
post-archive-validate: valid=true verdict=null
r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point] Mantle MUST run provider-backed Cargo-free one-shot and fixed-point proof stages with the explicit receipt-bound source-built Rust/native toolchain closure when that closure is supplied.
```

After recording post-archive evidence, pueue task 55 reran validation:

```text
post-evidence-validate: valid=true verdict=null
```
