# Shared action-result implementation evidence

Task-ID: I1 I2 I3 I4 I5 I6 I7 I8 I9 V1 V2 V3 V4 V5
Covers: build_correctness.shared_action_result_records, build_correctness.shared_action_result_admission, cache_substitution.shared_action_result_discovery

## Authority inventory

| Surface | Classification | Reason |
|---|---|---|
| Canonical derivation ATerm under the configured store prefix | identity input | `action_ref_for_derivation` hashes it with a domain-separated BLAKE3 action domain. |
| `mantle-action-result-v1` record | immutable signed claim | It binds the action, ordered outputs/object refs, PathInfo refs, receipt/policy/scan/signature refs, producer, publication policy, and non-claims. |
| PathInfo after ordinary store admission | authoritative output metadata | Reuse requires exact store-path linkage, complete castore content, and a signature verified by configured trust roots. |
| Candidate admission facts and `StrongReusePlan` | authoritative reuse decision | The pure core fails closed over every required fact and rejects conflicting admitted output sets. |
| Local CA mapping | advisory restart hint | It can locate a prior CA output but cannot prove action receipt, signatures, policy, scans, or object completeness. |
| Local/HTTP action index | advisory discovery | Presence and source authority cannot admit an output; every referenced immutable record is revalidated. |
| CAS object presence | advisory completeness input | Content presence alone does not identify an action, producer, receipt, policy, or PathInfo trust. |
| Executor success / producer label alone | unadmitted claim | Publication occurs only after output finalization, reference resolution, PathInfo signing, persistence, and export admission. |

## Implemented boundaries

- Pure identity/admission: `crates/crunch-action-result-core/src/lib.rs`
- ADR 0024 dependency guard: `crates/crunch-action-result-core/tests/architecture.rs`
- Local/HTTP action-result interface and shells: `crates/crunch-store/src/action_result.rs`
- Object/PathInfo probe and admitted-output persistence: `crates/crunch-store/src/handle.rs`
- Pre-execution lookup and post-admission publication: `crates/crunch-build/src/orchestrate.rs`
- Derivation/receipt/policy/signature adapter: `crates/crunch-build/src/action_result.rs`
- Typed policy and generated runtime JSON: `config/action-result-policy/`
- Plan/build human and JSON evidence: `src/build_plan.rs`, `src/build_cmd.rs`, `src/build_report.rs`
- Closed machine contract: `schemas/machine-contracts/build-json-report.schema.json`
- Weak metadata GC and CA mapping cleanup: `crates/crunch-store/src/gc.rs`

## Adversarial coverage

Positive coverage includes permutation-stable identities, duplicate deduplication, fully admitted reuse, local publication, HTTP publication, CA reuse without a CA mapping, and a clean HTTP client that fetches PathInfo/NAR content and performs zero executor calls.

Negative coverage includes stale action refs, object/ref drift, a non-CA output path differing from the path declared by the action, missing/bad signatures, signer/producer impersonation, policy mismatch, malformed/oversized records and indexes, incomplete and interrupted publication, no-clobber conflicts, corrupt HTTP records, timeouts, aggregate candidate overflow, offline remote suppression, index poisoning, conflicting admitted output sets, and GC proving candidate metadata does not root output objects.

## Final oracle checkpoint

- **Question:** Can index/CA/object metadata, a signed-but-wrong output, source order, publication interruption, or GC turn an unadmitted candidate into strong reuse or durable object retention?
- **Inspected evidence:** pure strong-admission predicates and permutation tests; cryptographic record and PathInfo verification; declared non-CA output-path binding; local hard-link/no-clobber and HTTP conditional-write code; fresh-client HTTP and CA-mapping-removal tests; failed-build and weak-metadata GC tests; final VibeThinker adversarial prompts.
- **Decision:** No blocking defect is supported. A compromised configured trust root can sign bad claims, but that is the explicit trust-policy boundary rather than metadata-presence admission. The independent review's concrete attack classes are all covered by the named negative tests.
- **Owner:** Mantle action-result admission and storage maintainers.
- **Next action:** Keep V5 unchecked until the unrelated repository-wide Tracey profile and existing package format/lint debt are repaired; do not weaken the action-result rails to make that baseline green.

## Validation transcript

Current exact command evidence is recorded in `validation.md` after the final affected-package, schema, formatting, lint, Cairn validation, Tracey, and stage-gate runs.
