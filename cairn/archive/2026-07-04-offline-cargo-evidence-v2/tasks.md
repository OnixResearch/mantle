## Implementation

- [x] [serial] I1 Define an offline Cargo evidence v2 data model with lockfile digest, source/vendor/toolchain identities, Cargo command shape, target/profile, network policy result, sidecar schema, and bounded non-claims. r[project_workflows.offline_cargo_digest_bound_evidence]
- [x] [serial] I2 Update `mantle.offlineCargoPackage` to write the v2 sidecar with deterministic BLAKE3 input digests or explicit narrower identity classes when full digests are unavailable. r[project_workflows.offline_cargo_digest_bound_evidence]
- [x] [serial] I3 Update build-report parsing to surface v2 evidence, label legacy v1 evidence, and emit deterministic diagnostics for malformed sidecars on offline Cargo outputs. r[project_workflows.offline_cargo_digest_bound_evidence]
- [x] [serial] I4 Thread vendor input identity from `vendor_src` into the sidecar and JSON report when vendored dependencies are declared. r[project_workflows.offline_cargo_digest_bound_evidence]
- [x] [serial] I5 Update docs and examples so evidence wording stays limited to `cargo-inside-mantle-sandbox` and names v2 digest-bound fields. r[project_workflows.offline_cargo_digest_bound_evidence]

## Verification

- [x] [serial] V1 Positive: a generated offline Cargo package writes v2 sidecar JSON and `mantle --json build` surfaces lockfile, source, optional vendor, toolchain, target/profile, network policy, and non-claims. r[project_workflows.offline_cargo_digest_bound_evidence]
- [x] [serial] V2 Positive: legacy v1 sidecars remain visible with a legacy evidence classification and no stronger digest-bound claim. r[project_workflows.offline_cargo_digest_bound_evidence]
- [x] [serial] V3 Negative: malformed JSON, wrong schema, wrong claim class, missing lockfile digest, invalid BLAKE3 hex, stale expected digest, and missing non-claims produce deterministic diagnostics instead of silent success. r[project_workflows.offline_cargo_digest_bound_evidence]
- [x] [serial] V4 Negative: an offline Cargo build that enables undeclared network access or omits source/vendor identity cannot produce accepted digest-bound evidence. r[project_workflows.offline_cargo_digest_bound_evidence]
- [x] [serial] V5 Run focused offline Cargo/build-report tests plus `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .`, proposal gate, design gate, and tasks gate for this change. r[project_workflows.offline_cargo_digest_bound_evidence]
