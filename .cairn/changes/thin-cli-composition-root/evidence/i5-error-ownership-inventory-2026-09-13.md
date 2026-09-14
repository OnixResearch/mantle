# I5 error ownership inventory

Change: `thin-cli-composition-root`
Task-ID: I5
Subject revision: `da8f7063a` plus the conversion recorded below.

## Question

I5 requires cores and application ports to return typed domain and capability
errors, and to map them to `RunError` only at the presentation boundary. Which
sites still return an untyped error, and how should they be measured?

## Inspected evidence

**Measurement correction.** A line-based `grep` for `Result<..., String>` counts
nested generics such as `Result<(BTreeMap<String, String>, Vec<Warning>), ShellError>`
and reports them as untyped errors. An AST-exact ast-grep pattern
(`fn $N($$$A) -> Result<$OK, String> { $$$B }`) over `crates/` replaces it.

Already satisfied:

- `RunError` appears in `crates/` exactly once, in a doc comment in
  `mantle-rust-plan-app/src/lib.rs` that documents the boundary the crate keeps.
- The contract, `mantle-rust-plan-core`, `mantle-rust-plan-app`,
  `mantle-portable-client-core`, `mantle-rust-plan-core`, `crunch-source-core`,
  `crunch-attestation-core`, and ten other cores have zero untyped returns.
- Every contract port returns `Result<_, CapabilityError>`.

Core files that still return an untyped error from at least one function:

| Core file |
| --- |
| `crates/crunch-rust-cache-core/src/lib.rs` |
| `crates/crunch-rust-cache-core/src/shared.rs` |
| `crates/crunch-rust-cache-core/src/wrapper.rs` |
| `crates/crunch-action-result-core/src/lib.rs` |
| `crates/crunch-hardware-simulation-core/src/digest.rs` |
| `crates/crunch-project-core/src/filegen.rs` |
| `crates/crunch-project-core/src/refresh.rs` |
| `crates/crunch-release-core/src/proof_audit.rs` |

Shell, adapter, and binary crates also contain such functions
(`crunch-store`, `crunch-build`, `crunch-eval`, `crunch-rust-cache`,
`crunch-wasm-component`, `crunch-rustc-wrapper`); they sit outside the core
boundary this task names.

## Decision

Convert one core at a time, smallest first, keeping the operator-visible text
byte-identical: `crunch-release-core/src/proof_audit.rs` was the first, where
`event_set_digest` now returns `ReleaseEvidenceError::Parse` and the caller
renders it into the same diagnostic string as before.

Add the checker rule that forbids untyped error returns in cores once the core
list is empty, so the rule never ships with a finding baseline.

## Owner

The cores own their error types; the shell maps them at the boundary.

## Next action

Convert `crunch-hardware-simulation-core/src/digest.rs`, then
`crunch-project-core` (two files), then `crunch-action-result-core`, then
`crunch-rust-cache-core` (three files, the largest).

## Non-claims

This inventory is a work list. It does not claim the remaining sites are wrong,
and it does not claim I5 is complete.
