## Design

### Goal

Prove the input trust policy capability against the already-accepted
`[depends:project_workflows.input_trust_policy]` requirement via a bounded offline proof
rail that emits versioned non-overclaiming evidence.

### Functional core / imperative shell

- **Core.** Trust-policy classification (verifier kind, trusted key set, required
  signer or quorum, binding of signature to fetched bytes or digest) and the
  accept/reject decision MUST live in a pure core with no I/O. The core receives
  fetched bytes (or their digest) plus trust evidence and returns accept or a
  classified rejection.
- **Shell.** The refresh shell fetches bytes, runs verification through the
  core, and writes lock entries only on accept. The rail driver stages fixtures
  with signed, unsigned, mismatched, untrusted, detached, and unsupported-verifier
  inputs and captures evidence.
- **Evidence is a pure render.** The rail renders a versioned JSON evidence
  object from the recorded verification decisions.

### Evidence shape

The emitted JSON evidence MUST include a stable schema version, per-input trust
decision (accepted or rejected-with-reason), the verified trust policy identity
(verifier kind and trusted key identity or fingerprint, not secret material),
binding assertion (signature bound to fetched bytes or digest), and explicit
non-claims (a hash-only input is content integrity, not signer trust or upstream
authenticity). Evidence MUST omit private key material, raw environment values,
and unbounded logs.

### Negative cases

- Missing, malformed, invalid, untrusted-key, detached-from-bytes, or
  unsupported-verifier trust material rejects the refresh before writing new
  lock entries; existing entries remain unchanged.
- A same-name-but-different-material key is rejected.
- A hash-only input (content hash but no trust policy) is reported as
  content-integrity-only and is not claimed as signer trust or upstream
  authenticity.

### Risks

- Trust verification may require a verifier implementation; the rail MUST use a
  bounded offline verifier fixture (e.g., a fixed test key pair) so no network or
  ambient keyring is needed.
- Redaction must ensure no secret key material reaches evidence or diagnostics.
