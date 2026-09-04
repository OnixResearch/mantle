# Tasks: Verify remote admission with Trellis

## Phase 1: Cross-repository contract and baseline

- [x] [serial] V1 Record the current Mantle remote-attempt state, report, fence, event, authorization, transition, retry, and application surfaces plus focused test output. r[remote_builds.trellis_admission_projection]
- [x] [serial] I1 Review and accept ADR 0060 for a Trellis-owned generic model, Mantle-owned projection, external proof evidence, and bounded claims. r[remote_builds.trellis_admission_model] r[remote_builds.trellis_admission_evidence_boundary]
- [x] [depends:add-fenced-attempt-admission-primitives] I2 Record the accepted Trellis requirement IDs, revision, source digest, verifier/toolchain identity, exported proof artifact contract, and non-claims. r[remote_builds.trellis_admission_model] r[remote_builds.trellis_admission_safety]
- [x] [depends:extend-nominal-types-to-trust-boundaries] I3 Reconcile the projection with admitted remote job, attempt, fence, event, digest, phase, and authorization types. r[remote_builds.trellis_admission_projection]
- [x] [depends:harden-remote-credential-boundary] [depends:add-nix-remote-service-gateway] [depends:add-evidence-driven-resource-policy] I4 Freeze the remote fact set used by the first proof profile after active remote contracts stabilize. r[remote_builds.trellis_admission_model]

## Phase 2: Trellis model and proof evidence

- [x] [serial] I5 Implement the paired Trellis `fenced_attempt_admission` spec, executable model, and proof functions for the closed safety set. r[remote_builds.trellis_admission_safety]
- [x] [parallel] V2 Run the Trellis positive and negative tests, full relevant Verus check, Tracey checks, and Trellis Cairn gates. Capture exact proof counts, assumptions, trusted boundaries, and source identity. r[remote_builds.trellis_admission_safety]
- [x] [serial] I6 Export the Trellis proof artifact through Kamacite and obtain the Valence acceptance receipt for the property role without changing Mantle runtime authority. r[remote_builds.trellis_admission_evidence_boundary]

## Phase 3: Mantle projection and parity

- [x] [serial] I7 Add a pure fail-closed projection from admitted Mantle remote-attempt facts to the Trellis model and reject every unmapped variant. r[remote_builds.trellis_admission_projection]
- [x] [serial] I8 Add the bounded complete parity matrix over phases, report kinds, fence classes, event classes, authorization classes, and result-linkage classes. r[remote_builds.trellis_admission_projection]
- [x] [serial] I9 Bind Mantle source, Trellis source, model, fixture, policy, verifier, proof artifact, Kamacite envelope, and Valence receipt identities in reviewable evidence. r[remote_builds.trellis_admission_evidence_boundary]
- [x] [serial] I10 Keep ordinary remote admission on Mantle's Rust core and keep Trellis/Kamacite/Valence processing outside runtime decision authority. r[remote_builds.trellis_admission_evidence_boundary]

## Phase 4: Positive and negative verification

- [x] [parallel] V3 Add positive fixtures for each accepted transition, idempotent duplicate, matching completion, and representable fence advance. r[remote_builds.trellis_admission_safety]
- [x] [parallel] V4 Add negative fixtures for stale, future, wrong-job, wrong-attempt, unauthorized, terminal, conflicting-event, mismatched-result, exhausted-fence, and unmapped-variant cases. r[remote_builds.trellis_admission_safety] r[remote_builds.trellis_admission_projection]
- [x] [parallel] V5 Add mutation fixtures that change one Mantle transition, one Trellis mapping, one proof identity, one assumption, and one Valence role and prove each mismatch fails closed. r[remote_builds.trellis_admission_projection] r[remote_builds.trellis_admission_evidence_boundary]
- [x] [parallel] V6 Add a claim-text guard that rejects persistence, transport, cryptography, worker, liveness, whole-build, and release overclaims. r[remote_builds.trellis_admission_claim_boundary]

## Phase 5: Documentation and lifecycle

- [x] [serial] I11 Document the modeled fields, property list, Mantle-to-Trellis mapping, evidence chain, update procedure, and all non-claims. r[remote_builds.trellis_admission_claim_boundary]
- [x] [serial] V7 Run focused `crunch-build` remote-attempt tests, projection/parity tests, Trellis-sidecar binding tests, remote negative rails, and first-party quality checks. Record exact positive and negative summaries. r[remote_builds.trellis_admission_projection] r[remote_builds.trellis_admission_evidence_boundary]
- [x] [serial] V8 Run Cairn validation, Tracey coverage, all three change gates, and relevant Nix checks in Mantle and the paired Trellis change. r[remote_builds.trellis_admission_claim_boundary]
