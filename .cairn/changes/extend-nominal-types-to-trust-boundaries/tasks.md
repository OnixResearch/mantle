# Tasks: Extend nominal types to trust boundaries

## Phase 1: Baseline and shared rules

- [x] [serial] V1 Record current public Rust call sites, accepted JSON bytes, canonical BLAKE3 identities, and focused test results before core changes. r[build_correctness.nominal_boundaries.compatibility]
- [x] [serial] I1 Define crate-local constructor rules for bounded identifiers, safe path classes, unit-bearing limits, and lowercase BLAKE3 values. r[build_correctness.nominal_boundaries.admission]
- [x] [serial] I2 Add explicit structural-wire to admitted-core conversion seams where validation must retain several diagnostics. r[build_correctness.nominal_boundaries.admission]
- [x] [parallel] V2 Add positive constructors and negative empty, oversized, control-bearing, malformed, zero, overflow, and direct-Serde fixtures. r[build_correctness.nominal_boundaries.admission]

## Phase 2: Bootstrap admission

- [x] [serial] I3 Make bootstrap `Blake3Hex` construction fallible and add selected StageX digest roles without changing serialized text. r[build_correctness.nominal_boundaries.digests]
- [x] [serial] I4 Admit StageX stage, artifact, executable-authorization, absolute-path, timeout, byte-limit, and parallel-job values before graph checks. r[build_correctness.nominal_boundaries.identities] r[build_correctness.nominal_boundaries.units_and_paths]
- [x] [serial] I5 Migrate bootstrap lineage declarations to role-specific IDs and resolve graph endpoints into typed node roles. r[build_correctness.nominal_boundaries.identities]
- [x] [parallel] V3 Add positive StageX and lineage fixtures plus negative role substitution, unresolved reference, malformed digest, unsafe path, zero limit, and excessive limit fixtures. r[build_correctness.nominal_boundaries.identities] r[build_correctness.nominal_boundaries.units_and_paths]

## Phase 3: Remote protocol identities

- [x] [depends:harden-remote-credential-boundary] I6 Admit bounded remote request, protocol-session, endpoint, output, and transfer identities without imposing a new digest-only wire grammar. r[build_correctness.nominal_boundaries.identities]
- [x] [depends:harden-remote-credential-boundary] I7 Consume the credential-owned ticket, verifier, validity, use, build-time, and upload-limit types through narrow protocol adapters. r[build_correctness.nominal_boundaries.units_and_paths]
- [x] [parallel] V4 Add positive protocol round trips and negative request/session exchange, endpoint mismatch, malformed wire, and credential-type substitution tests. r[build_correctness.nominal_boundaries.identities]

## Phase 4: Evidence and frontend admission

- [x] [serial] I8 Admit content-bound repository, requirement, release, specification-path, evidence-path, and BLAKE3 values while preserving bounded multi-issue reports. r[build_correctness.nominal_boundaries.admission] r[build_correctness.nominal_boundaries.digests]
- [x] [serial] I9 Represent content-bound digest domains as checked typed values or a checked sum type instead of independent domain and raw digest values. r[build_correctness.nominal_boundaries.digests]
- [x] [serial] I10 Admit frontend spec, validator, artifact, target, build-root, and spec-hash values before attestation construction. r[build_correctness.nominal_boundaries.identities] r[build_correctness.nominal_boundaries.digests]
- [x] [parallel] V5 Add positive and negative content-bound and frontend fixtures for wrong roles, malformed paths, malformed digests, unsupported algorithms, unsupported validators, and binding mismatches. r[build_correctness.nominal_boundaries.admission]

## Phase 5: Build helper boundaries

- [x] [serial] I11 Replace selected fetch URL, Git revision, archive path, and output-directory parameter groups with checked values and typed request records. r[build_correctness.nominal_boundaries.units_and_paths]
- [x] [serial] I12 Add derivation-name, output-name, logical-prefix, provisional-path, final-path, and derivation-key types where content-addressed planning or rewriting depends on role. r[build_correctness.nominal_boundaries.identities] r[build_correctness.nominal_boundaries.units_and_paths]
- [x] [parallel] V6 Add positive fetch, CA planning, rewrite, and pipeline tests plus negative swapped-role, wrong-prefix, unsafe-path, and malformed-revision cases. r[build_correctness.nominal_boundaries.identities]

## Phase 6: Compatibility, policy, and closeout

- [x] [serial] I13 Project admitted values through accepted wire DTOs and compatibility adapters. Preserve field names, scalar spellings, canonical bytes, and BLAKE3 identities. r[build_correctness.nominal_boundaries.compatibility]
- [x] [serial] I14 Add reviewed nominal-domain declarations for migrated scopes after the generated Cairn and Octet policy inputs are current. r[build_correctness.nominal_boundaries.guard]
- [x] [serial] I15 Document each admitted boundary, migration adapter, and local non-claim. r[build_correctness.nominal_boundaries.claim_boundary]
- [x] [parallel] V7 Add compile-pass same-role fixtures and compile-fail cross-role fixtures for IDs, digests, paths, and unit-bearing limits. r[build_correctness.nominal_boundaries.guard]
- [x] [serial] V8 Run `nix develop -c cargo test -p crunch-bootstrap-core`, `nix develop -c cargo check -p crunch-bootstrap-core --target wasm32-unknown-unknown`, `nix develop -c cargo test -p crunch-build`, `nix develop -c cargo test -p crunch-pipeline`, `nix develop -c cargo test -p crunch-release-core`, and `nix develop -c cargo check -p crunch-release-core --target wasm32-unknown-unknown`. r[build_correctness.nominal_boundaries.compatibility]
- [x] [serial] V9 Run `nix develop -c cargo test -p mantle --bin mantle frontend_artifact_spec::` and `nix develop -c cargo test -p mantle --bin mantle remote_build::`. Record positive and negative summaries. r[build_correctness.nominal_boundaries.admission]
- [x] [serial] V10 Run `./scripts/check-first-party-tigerstyle.sh` and `./scripts/check-first-party-quality.sh`. r[build_correctness.nominal_boundaries.guard]
- [x] [serial] V11 Run Cairn validation, proposal gate, design gate, tasks gate, Tracey coverage, and the relevant Nix checks. Record the exact command output. r[build_correctness.nominal_boundaries.claim_boundary]

## Verification coverage

- `Scenario: Valid structural input admits nominal values` -> I1, I2, V2
- `Scenario: Invalid values cannot bypass admission` -> I1, I2, V2
- `Scenario: Same-format digest roles remain distinct` -> I3, I9, V5, V7
- `Scenario: Graph references retain resolved roles` -> I4, I5, V3
- `Scenario: Units and path authorities cannot be exchanged` -> I4, I7, I11, I12, V6, V7
- `Scenario: Accepted wire identity remains stable` -> V1, I13, V8, V9
- `Scenario: Primitive regressions fail policy` -> I14, V7, V10
- `Scenario: Nominal claims remain local` -> I15, V11
