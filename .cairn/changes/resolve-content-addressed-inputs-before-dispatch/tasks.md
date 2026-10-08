# Tasks: Resolve content-addressed inputs before dispatch

Tasks are completed only when their evidence or exercised behavior is recorded below.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Record the baseline in an isolated worktree: derivation-path, output-path, and action-ref goldens for input-addressed fixtures; current CA mapping, rewrite, and cache-check behavior; focused `crunch-build` and `crunch-store` test output. r[mantle.ca_input_resolution.resolved_identity] Evidence: `evidence/baseline-2026-10-01.md`.
- [x] [serial] T1.2 Define the resolution function, the resolved identity domain, the realisation record schema, the trust policy, limits, and the failure catalog. r[mantle.ca_input_resolution.resolved_derivation] r[mantle.ca_input_resolution.realisation_records] Evidence: `design.md` and `specs/ca-input-resolution/spec.md`.
- [x] [serial] T1.3 Record the resolved-identity and realisation decisions in an ADR with an index row in `adr/README.md`. r[mantle.ca_input_resolution.resolved_identity] Evidence: `adr/0098-resolve-content-addressed-inputs-before-dispatch.md`.

## Phase 2: Pure core

- [x] [serial] T2.1 Implement pure resolution and resolved identity, leaving derivations without CA inputs unchanged. r[mantle.ca_input_resolution.resolved_derivation] r[mantle.ca_input_resolution.resolved_identity] Evidence: `evidence/after-2026-10-01.md` (pure resolver, 3 passed).
- [x] [parallel] T2.2 Add positive core fixtures: substitution in arguments and environment, edge replacement, idempotence, and two unresolved forms sharing one resolved form. r[mantle.ca_input_resolution.resolved_derivation] Evidence: pure resolver fixtures in `evidence/after-2026-10-01.md`.
- [x] [parallel] T2.3 Add negative core fixtures: an unrealized input, a wrong-domain identity, and over-limit resolution. r[mantle.ca_input_resolution.negative_controls] Evidence: pure resolver fixtures in `evidence/after-2026-10-01.md`.

## Phase 3: Shell integration

- [x] [serial] T3.1 Resolve before cache lookup and dispatch, and key cache lookup, CA mappings, and action refs on resolved identity for CA-dependent derivations. r[mantle.ca_input_resolution.resolved_identity] r[mantle.ca_input_resolution.early_cutoff] Evidence: transitive cutoff in `evidence/after-2026-10-01.md`.
- [x] [serial] T3.2 Persist and admit signed realisation records under the PathInfo trust policy and publish them through existing substitution and action-result sources. r[mantle.ca_input_resolution.realisation_records] Evidence: clean HTTP client and forged-intermediate fixtures in `evidence/after-2026-10-01.md`.
- [x] [serial] T3.3 Bind dynamic-plan placeholders on CA unit outputs during resolution and remove the registration-time rejection. r[mantle.ca_input_resolution.dynamic_plan_binding] Evidence: native CA unit placeholder fixture and CA-resolved IA registry-output red/green regression in `evidence/after-2026-10-01.md`.
- [x] [serial] T3.4 Stop applying post-build input rewrites to resolved builds and remove the rewrite path once no caller remains. r[mantle.ca_input_resolution.resolved_derivation] Evidence: resolved input dispatch in the transitive fixture; no `collect_ca_input_rewrites` or `apply_input_rewrites` source caller remains.
- [x] [parallel] T3.5 Add shell negative controls: conflicting realisations with nondeterminism evidence, unsigned and untrusted records, and unrealized inputs. r[mantle.ca_input_resolution.negative_controls] Evidence: conflict, forged child, wrong full key, mapping-only, and unrealized fixtures in `evidence/after-2026-10-01.md`.

## Phase 4: Verification and documentation

- [x] [serial] T4.1 Run the early-cutoff fixture and record zero dependent executions with resolved identities in the report. r[mantle.ca_input_resolution.early_cutoff] Evidence: three-node cutoff in `evidence/after-2026-10-01.md`.
- [x] [serial] T4.2 Run a clean-client fixture that reuses signed realisations and PathInfo without execution. r[mantle.ca_input_resolution.realisation_records] Evidence: clean three-node HTTP client in `evidence/after-2026-10-01.md`.
- [ ] [serial] T4.3 Prove identity goldens for derivations without CA inputs are unchanged. r[mantle.ca_input_resolution.resolved_identity]
- [x] [serial] T4.4 Document resolution, realisation records, trust, and non-claims in the build and cache documentation. r[mantle.ca_input_resolution.realisation_records] Evidence: `README.md` build/cache section, ADR 0098, and corrected `docs/dependency-audit.md` publication distinction.
- [ ] [serial] T4.5 Run focused `crunch-build`, `crunch-store`, and `crunch-pipeline` suites before and after the change, strict Clippy for touched first-party packages, Cairn validation, and the proposal, design, and tasks gates. Preserve exact output in `evidence/`. r[mantle.ca_input_resolution.negative_controls]
- [ ] [serial] T4.6 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.ca_input_resolution.early_cutoff]
