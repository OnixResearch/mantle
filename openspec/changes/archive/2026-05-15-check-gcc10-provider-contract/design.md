## Context

`bootstrap/gcc-10.ncl` already contains a bounded provider shape: it configures GCC 10.5.0 for C and C++, builds `all-gcc` and `all-target-libgcc`, installs both, checks compiler entrypoints and `libgcc.a`, and compiles C and C++ smoke programs. The parity report still treats `gcc.10` as a blank partial row with `EvidenceCheck::None`.

## Goals / Non-Goals

**Goals:**
- Make the `gcc.10` partial row evidence-backed and self-describing.
- Fail closed if the receipt is missing, malformed, or claims markers that no longer exist in `bootstrap/gcc-10.ncl`.
- Preserve the claim boundary: contract-only evidence is not native/full GCC 10 correctness.

**Non-Goals:**
- Do not prove full GCC 10 native correctness.
- Do not build the full source chain as part of this change.
- Do not change CLI flags or JSON schema.

## Decisions

### 1. Use a repo-local JSON receipt with required markers

**Choice:** Add `bootstrap/evidence/gcc-10-provider-contract.json` with schema, derivation, `contract-only` status, required derivation markers, and explicit parity effect.

**Rationale:** This matches the existing GCC 4.7 evidence pattern and keeps the receipt reviewable without depending on a long-running bootstrap build.

**Alternative:** Only hard-code the marker list in Rust. Rejected because the evidence artifact should be inspectable and reusable in parity reports.

### 2. Reuse the parity evidence-check pattern

**Choice:** Add a `Gcc10ProviderContract` evidence check that validates receipt fields and verifies every required marker appears in `bootstrap/gcc-10.ncl`.

**Rationale:** The check is deterministic, fast, and catches both missing evidence and derivation drift.

**Alternative:** Mark the row complete when the receipt validates. Rejected because the receipt proves provider shape only, not full source-chain/native correctness.

## Risks / Trade-offs

**Marker brittleness** → Use meaningful configure/build/install/smoke strings, not incidental whitespace-only fragments.

**Overclaiming parity** → Keep `expected_complete = false`, row status `partial`, and receipt `parity_effect` explicit.

## Validation Plan

- `cargo test --bin mantle bootstrap_parity::tests -- --nocapture` or the repo's equivalent targeted parity unit tests.
- `cargo test --test bootstrap_parity_cli -- --nocapture`.
- `cargo run --bin mantle -- --json bootstrap parity-report` confirms `gcc.10` remains partial with evidence-backed notes and no evidence failure.
- `openspec validate check-gcc10-provider-contract --strict` and `openspec validate --all --strict`.
- `cargo fmt --check` and `git diff --check`.
