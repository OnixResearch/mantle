# Compiled-eval Tracey backfill map

Task-ID: compiled-eval-backfill-map
Covers: verification_evidence.tracey_coverage_readiness

## Question

Which `compiled-eval.*` legacy OpenSpec requirements can be safely referenced from the Mantle Tracey bridge without claiming a shipped compiled evaluator or full future semantic support?

## Inspected evidence

- `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/tasks.md`
- `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/design.md`
- `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/proposal.md`
- `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/specs/compiled-eval-backends/spec.md`
- `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/evidence/V1-eval-tests.md`
- `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/evidence/V2-benchmark-compare.md`
- `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/evidence/V3-openspec-gates.md`
- `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/evidence/V4-unsupported-subset.md`
- `crates/crunch-eval/src/backend.rs`
- `crates/crunch-eval/src/lib.rs`
- `crates/crunch-eval/src/cranelift_proto.rs`
- `crates/crunch-eval/Cargo.toml`

## Mapping

| Requirement IDs | Bridge verb | Evidence basis | Claim boundary |
| --- | --- | --- | --- |
| `compiled-eval.backend-boundary`, `.default` | `impl` + `verify` | `backend.rs` keeps `NickelBackend` default; `lib.rs` public helpers call `default_backend()`; archived V1/V3 test/gate evidence. | Current default interpreter path and private backend seam only. |
| `compiled-eval.backend-boundary.swap` | `related` | Archived design decision 1 keeps future backends behind `crunch-eval`. | Future swap support is a design constraint; no future backend implementation claim. |
| `compiled-eval.profiling-gate`, `.no-evidence`, `.evidence` | `related` | Archived V2/design records benchmark/profiling gate and tabled status. | Process/evidence linkage only; no current priority or shipped backend claim. |
| `compiled-eval.benchmark-guardrail`, `.recorded`, `.regression` | `related` | Archived V2/design records baseline bundle and non-eval guardrail drift. | Prototype remains non-shipping; no promotion claim. |
| `compiled-eval.cranelift-first`, `.initial`, `.llvm-later` | `related` | Archived design decision 4 records Cranelift-first and LLVM-later rationale. | Directional prototype rationale only; LLVM remains optional future work. |
| `compiled-eval.private-backend-seam`, `.callers`, `.no-leak` | `impl` + `verify` | `backend.rs` private trait/request carrier; `lib.rs` public helpers stay source-compatible; archived I3/V1/V3 evidence. | Private seam and no public compiler-backend API. |
| `compiled-eval.cranelift-prototype-subset`, `.default`, `.unsupported` | `impl` + `verify` | `Cargo.toml` gates Cranelift dependencies behind `cranelift-proto`; `cranelift_proto.rs` parses only flat derivation fields; `backend.rs` rejects import paths; V1/V4 evidence records tests/source audit. | Feature-gated prototype subset only. |
| `compiled-eval.future-semantics`, `.contracts`, `.shape` | `related` | Archived design decision 7 and V4 evidence keep broader semantics outside the prototype and require future equivalence work. | Non-claim: full future semantic equivalence is not implemented. |

## Decision

Add bounded bridge references in `tools/tracey_refs.rs` for all 21 `compiled-eval.*` IDs. Use `impl`/`verify` only for current code-backed default boundary, private seam, and feature-gated subset behavior. Use `related` for profiling gates, benchmark guardrails, Cranelift-first direction, future swap constraints, and future semantic-equivalence requirements.

This should reduce Tracey missing count without implying that compiled evaluation is shipped, promoted, complete, or currently prioritized.

## Owner

Mantle maintainer / current agent for `tracey-coverage-readiness-backfill`.

## Next action

Run `cairn tracey coverage --root . --json` and confirm the `compiled-eval` missing group disappears. If Tracey still lists any `compiled-eval.*` IDs, keep those IDs as explicit legacy debt rather than upgrading non-claims to implementation claims.
