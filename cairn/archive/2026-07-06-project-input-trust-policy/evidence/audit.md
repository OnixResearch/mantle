# Audit: Input Trust Policy Surface

Audited against `r[project_workflows.input_trust_policy]` requirement clauses and
the change's own `r[project_workflows.input_trust_policy_proof_rail]` requirement.

## Source Files Audited

- `crates/crunch-project-core/src/trust.rs` (400 lines)
- `crates/crunch-project-core/src/soundness.rs` (trust policy items)
- `crates/crunch-project/src/refresh_adapter.rs` (verify_trust_policy usage)
- `src/project_resolve.rs` (verify_project_trust_policy, read_project_trust_signature)
- `src/pin_import.rs` (trust policy in import)
- `tests/project_refresh_cli.rs` (trust policy tests)

## Core Trust Implementation

### Types and Policies
- ✅ `InputTrustPolicy` with `verifier` (`TrustVerifierKind`), `signatures`
  (`Vec<TrustSignatureRef>`), `required_signers` (optional), `required_quorum`
  (optional), `digest_binding` (`TrustDigestBinding`).
- ✅ `TrustVerifierKind::Ed25519Detached` — supported verifier.
- ✅ `TrustDigestBinding::ContentHash` — signature payload bound to hash.
- ✅ `TrustSubject` with `TrustSubjectKind::Input` / `TrustSubjectKind::Patch`.
- ✅ `TrustSignatureRef::LocalFile { path }` — local signature file reference.
- ✅ `VerifiedTrustFact` with bound bytes, verification kind, signer identity.
- ✅ `evaluate_trust_policy()` — pure core decision logic.

### Shell Verification
- ✅ `verify_project_trust_policy()` in `src/project_resolve.rs` reads signature
  files, constructs payload, calls `evaluate_trust_policy()`.
- ✅ `trust_signature_payload()` constructs Ed25519 digest-binding payload.
- ✅ `read_project_trust_signature()` handles `LocalFile` and
  `LocalFilePath` variants.
- ✅ `TRUST_KEYPAIR` constant for test key material.

### Refresh Integration
- ✅ `verify_trust_policy()` method on `RefreshResolver` trait in refresh_adapter.
- ✅ `LiveProjectResolver` in `src/project_resolve.rs` implements it.
- ✅ Trust policy checked before lock entry write in refresh flow.

### Soundness Integration
- ✅ `check_project_soundness()` in soundness.rs checks trust policy
  compatibility during static project soundness.
- ✅ `PROJECT_INPUT_TRUST_NON_CLAIM` constant.

## Coverage Against Requirement

### Trusted Input Refresh Accepted
- ✅ Core: `evaluate_trust_policy()` returns `Ok(facts)` for valid signature.
- ✅ Shell: `verify_project_trust_policy()` returns `Ok(facts)`.
- ✅ Test: `refresh_trusted_file_input_writes_lock_evidence_and_show_reports_claim`.

### Missing/Invalid Trust Blocks Lock Update
- ✅ Core: returns error for missing signature file, malformed, invalid key.
- ✅ Shell: `refresh_inputs()` propagates error → lock entry not written.
- ✅ Test: `refresh_trusted_file_input_with_wrong_signature_rejects_lock_write`.

### Hash-Only Is Not Signed Evidence
- ✅ `PROJECT_INPUT_TRUST_NON_CLAIM`: "project input trust policy does not prove..."
- ✅ `PROJECT_INPUT_TRUST_NON_CLAIM` wired in soundness check output.
- ⚠️ No dedicated test asserting hash-only non-claim in verification flow.

### Same-Name Different-Material Key
- ⚠️ Core key comparison uses explicit `[u8; 32]` public key bytes.
- ⚠️ No test for same-name-different-material key rejection.

### Detached-from-Bytes Rejection
- ✅ `TrustDigestBinding::ContentHash` constructs payload from hash bytes.
- ✅ Wrong hash → digest mismatch → verification fails.
- ⚠️ No dedicated test for detached signature (bytes vs payload mismatch).

### Unsupported Verifier
- ⚠️ `TrustVerifierKind` has only `Ed25519Detached`. Unsupported variants
  (e.g., `Pgp`, `Minisign`) would need to be defined. The core would reject
  them at `evaluate_trust_policy()` time.
- ⚠️ No test for unsupported verifier kind.

## Gaps for Proof Rail

| Feature | Code | Test |
|---|---|---|
| Positive: trusted refresh accepted | ✅ | ✅ |
| Negative: missing signature blocks | ✅ | ✅ |
| Negative: wrong signature blocks | ✅ | ✅ |
| Negative: malformed signature blocks | ✅ | ❌ |
| Negative: untrusted key blocks | ✅ | ❌ |
| Negative: same-name-different-material key | ⚠️ | ❌ |
| Negative: detached from bytes | ⚠️ | ❌ |
| Negative: unsupported verifier | ❌ | ❌ |
| Hash-only non-claim | ✅ | ❌ |
| Redaction (no secret leakage) | ❌ | ❌ |
| Versioned evidence record | ❌ | ❌ |
| Composed proof rail | ❌ | ❌ |

**Gaps to address:**
1. Composed offline proof rail with test key fixture
2. Versioned, redacted evidence record
3. Negative tests for 6 trust-failure cases (missing, malformed, invalid,
   untrusted, detached, unsupported)
4. Hash-only non-claim assertion
5. Redaction assertion