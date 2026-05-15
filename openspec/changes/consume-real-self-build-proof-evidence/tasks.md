## Phase 1: Evidence contract

- [ ] [serial] Define the compact real self-build parity evidence descriptor for `crunch.self-build`, including schema/version, release id, selected provider kind, proof/verify/sandbox/summary BLAKE3 linkage, and bounded claim text.
- [ ] [depends:evidence-contract] Implement validation that mirrors the real-proof receipt checker invariants for deterministic proof workflow/version, `self-rebuild-match`, two clean roots, artifact digest equality, provider-kind match, sandbox profile support, and verify `eligible`.

## Phase 2: Parity integration

- [ ] [depends:evidence-validation] Wire the validated descriptor into `bootstrap parity-report` row construction for `crunch.self-build`, surfacing selected provider kind, proof evidence digest, and fail-closed diagnostics.
- [ ] [depends:parity-row] Preserve axis gating so bounded legacy-provider self-build proof evidence does not complete Guix or StageX parity without source-root or StageX lineage evidence.

## Phase 3: Regression coverage

- [ ] [parallel] Add positive parity/unit coverage for a valid bounded real self-build proof descriptor.
- [ ] [parallel] Add negative coverage for missing descriptor, malformed digest linkage, unsupported workflow version, provider-kind mismatch, missing/unsupported sandbox evidence, reused proof roots, and ineligible release verify receipt.
- [ ] [parallel] Add CLI regression for `crunch --json bootstrap parity-report` output fields and blocker behavior.

## Phase 4: Verification and docs

- [ ] [depends:tests] Update operator docs/README only if the parity evidence descriptor is operator-facing.
- [ ] [depends:tests] Run targeted Rust parity tests, CLI parity tests, `git diff --check`, and `openspec validate consume-real-self-build-proof-evidence --strict`.
