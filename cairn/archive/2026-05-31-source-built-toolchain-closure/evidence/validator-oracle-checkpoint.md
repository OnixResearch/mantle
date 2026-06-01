# Oracle checkpoint: validator task completion scope

Task-ID: I1-oracle
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Question

Does checked task I1 overclaim relative to the validator code and tests that exist now?

## Inspected evidence

Checked task text in `cairn/changes/source-built-toolchain-closure/tasks.md` says I1 defines a typed toolchain-closure manifest and pure validator for roles, BLAKE3 digests, source identities, build receipt identities, trust classifications, seed exceptions, and closure policy digest.

Inspected implementation in `src/source_toolchain_closure.rs`:

- Typed manifest structures: `ToolchainClosureManifest`, `ToolchainClosureMember`, `ToolchainSourceIdentity`, `ToolchainBuildReceiptIdentity`, and `ToolchainSeedException`.
- Roles and trust classifications: `ToolchainRole` and `ToolchainTrust` with serde `kebab-case` wire names.
- Entry point: `validate_toolchain_closure_manifest(...)` normalizes in-memory data and returns `ToolchainClosureValidation` with policy digest and counts.
- Role coverage: `validate_required_roles(...)` requires `Rustc`, `Linker`, `CCompiler`, and `Sysroot`.
- BLAKE3 digest shape: `validate_blake3_hex(...)` enforces lowercase hex with `blake3::OUT_LEN * 2` characters.
- Source and receipt identity checks: `validate_optional_member_identities(...)`, `validate_source_built_member(...)`, `validate_source_identity(...)`, and `validate_receipt_identity(...)` validate identities when present and require source/receipt for source-built members.
- Seed exception checks: `validate_seed_member(...)` requires matching `(name, content_digest_blake3)` in validated seed exceptions.
- Closure policy digest: `digest_normalized_manifest(...)` hashes a sorted normalized manifest under context `mantle-source-built-toolchain-policy-digest-v1` using BLAKE3.
- Functional core boundary: module uses in-memory structs and pure validation/hashing; filesystem reads are outside this module in `src/cargo_free_self_build.rs::load_source_built_toolchain_closure(...)`.

Inspected focused tests in `src/source_toolchain_closure.rs`:

- `valid_manifest_yields_stable_order_independent_policy_digest`
- `validator_rejects_source_built_member_without_receipt`
- `validator_rejects_seed_member_without_seed_exception`
- `validator_accepts_explicit_seed_exception`
- `validator_rejects_invalid_optional_source_on_seed_member`
- `validator_rejects_invalid_optional_receipt_on_seed_member`
- `validator_rejects_invalid_digest_shape`
- `validator_rejects_missing_required_role`
- `validator_rejects_duplicate_member_identity`
- `validated_closure_status_keeps_claim_disabled_until_enforcement_lands`
- `absent_closure_status_preserves_current_non_claim`

## Decision

I1 is narrow enough to remain checked. It proves a typed in-memory manifest validator and deterministic policy digest. It does not prove host-tool leakage enforcement, stage policy digest threading, real source-root/seed provider integration, or an end-to-end source-built toolchain closure proof; those remain unchecked tasks.

## Next action

Keep I2 unchecked until host-tool leakage enforcement exists and negative tests prove it. Keep `claim=false` and `not-source-built-toolchain-closure` for validated manifests until enforcement and real closure proof land.
