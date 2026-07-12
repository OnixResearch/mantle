## Phase 1: Production Cairn handoff

- [x] [serial] Normalize production Cairn handoff inputs and measure referenced artifact bytes in the shell. r[mantle.release_provenance.cairn_evidence_handoff.measured_inputs]
- [x] [serial] Invoke the pure Cairn handoff validator from release assembly and `mantle release verify` before pass evidence is admitted. r[mantle.release_provenance.cairn_evidence_handoff.production_wiring]
- [x] [serial] Bind the validation receipt to the same release bundle and fail required profiles when validation is absent or bypassed. r[mantle.release_provenance.cairn_evidence_handoff.bypass_protection]

## Phase 2: Source-root capability and hermetic policy

- [x] [serial] Replace the permanently unavailable source-root path with a bounded implementation or remove it from executable command discovery behind an explicit unsupported-capability report. r[mantle.build_correctness.source_root_capability]
- [x] [serial] Keep source-root planning pure and capability probing, filesystem access, and execution in the shell. r[mantle.build_correctness.source_root_capability.boundary]
- [x] [serial] Require strict hermetic mode for Onix release profiles and prevent practical-mode receipts from satisfying strict release evidence. r[mantle.build_correctness.onix_release_strict_hermeticity]

## Phase 3: Dependencies, fixtures, and docs

- [ ] [serial] Require the archived Cairn `authenticate-stack-provenance-inputs` receipt before accepting authenticated handoff evidence. r[mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency]
  - BLOCKED: the external change is active with metadata status `blocked`, has 0/13 tasks complete, and has no archive entry. Mantle rejects authenticated promotion. See `evidence/external-authentication-blocker.md`.
- [x] [parallel] Add positive fixtures covering production handoff validation, implemented or honestly unsupported source-root capability, and clean strict execution. r[mantle.release_provenance.cairn_evidence_handoff.fixtures.positive] r[mantle.build_correctness.hermetic_handoff.fixtures.positive]
- [x] [parallel] Add negative fixtures for validator bypass, tampered bytes, stale policy, unavailable advertised command, host influence, and practical-mode promotion. r[mantle.release_provenance.cairn_evidence_handoff.fixtures.negative] r[mantle.build_correctness.hermetic_handoff.fixtures.negative]
- [x] [parallel] Document production wiring, capability reporting, hermetic profile selection, and bounded claims. r[mantle.release_provenance.cairn_evidence_handoff.docs] r[mantle.build_correctness.hermetic_handoff.docs]

## Phase 4: Verification

- [x] [serial] Run focused positive and negative release-handoff, source-root capability, and hermeticity tests. r[mantle.release_provenance.cairn_evidence_handoff.final_validation]
  - PASS: core, binary, and production CLI suites cover measured bytes, tampering, manifest/bundle reuse, Onix strict admission, honest capability reporting, and source-root regressions. See `evidence/validation.md`.
- [x] [serial] Add a checked-in CI workflow that runs only `nix flake check`. r[mantle.release_provenance.cairn_evidence_handoff.flake_check_ci]
- [ ] [serial] Run `nix flake check` and Cairn validation/gates before sync and archive. r[mantle.release_provenance.cairn_evidence_handoff.final_validation]
  - CLOSEOUT BLOCKED: final `nix flake check` fails the pre-existing bootstrap blocker inventory (40 findings; 396 evidence-backed suppressions); an earlier attempt also exposed untouched repository-wide format drift. The authenticated Cairn dependency is unarchived. Cairn gates pass, but this task stays unchecked and no sync/archive is performed.
