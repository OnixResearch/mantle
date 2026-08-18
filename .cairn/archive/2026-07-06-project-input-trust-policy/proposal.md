## Why

The `project-workflows` spec already accepts
`[depends:project_workflows.input_trust_policy]`: Mantle MUST support explicit trust
policy for project inputs and patches (verifier kind, signature material refs,
trusted public key identities or fingerprints, required signer or quorum, binding
to fetched bytes or digest), and required trust policy MUST be verified before a
lockfile refresh writes new source or patch hashes. Partial surface exists
(`crates/crunch-project/src/refresh_adapter.rs`, `src/project_resolve.rs`,
`src/pin_import.rs`), but the trust-gates-refresh behavior is not proven as a
bounded offline proof rail with versioned, redacted, non-overclaiming evidence.

## What Changes

- Audit the existing trust-policy surface against every scenario clause of the
  accepted requirement.
- Provide a bounded local offline proof rail that exercises input trust policy
  verification during refresh: accept only when fetched bytes match the content
  hash and present valid trust evidence from the configured signer set; reject
  before writing new lock entries on missing, malformed, invalid, untrusted,
  detached, or unsupported-verifier trust material.
- Emit a versioned, redacted, non-overclaiming evidence record that identifies
  the verified trust policy without exposing secret key material and states that
  a hash-only input is content integrity, not signer trust or upstream
  authenticity.
- Add negative cases: missing/invalid/untrusted/detached/unsupported-verifier
  trust blocks the lock update and leaves existing entries unchanged; a
  hash-only input is not signed evidence.

## Impact

- **Files**: `crates/crunch-project/src/refresh_adapter.rs`,
  `crates/crunch-project/src/lib.rs`, `src/project_resolve.rs`, `src/pin_import.rs`,
  a bounded offline proof rail, and an evidence-render helper.
- **Testing**: positive trusted-refresh-accepted case, negative trust-failure
  cases, hash-only non-claim case, redaction assertion (no secret leakage), and
  the Cairn gates.

## Out of Scope

- Designing a new signature scheme or verifier; only wiring existing verifiers.
- Treating trust policy as build success or release reproducibility proof.
- Changes to the accepted `input_trust_policy` requirement text.
