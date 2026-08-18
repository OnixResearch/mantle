## ADDED Requirements

### Requirement: Remote outputs use a single trust-admission pipeline

r[remote_builds.output_trust_admission_pipeline] Mantle MUST admit remote build outputs only through a single output-trust pipeline that checks requested output identity, logical store prefix, PathInfo signatures, signer key material, object refs, artifact attestations, producer policy, revocation/expiration state, and requested claim strength. Builder tickets, trusted-client entries, coordinator assignment, or transport authentication MUST authorize only resource access and MUST NOT by themselves admit outputs.

#### Scenario: trusted output evidence is imported

GIVEN a remote builder returns PathInfo and artifact-attestation evidence signed by key material trusted by the client
AND the returned output identity, object refs, store prefix, producer policy, and requested output names match the concrete request
WHEN Mantle performs output admission
THEN Mantle MAY import and export the output
AND the build report MUST identify the verified trust basis without exposing private key paths or bearer ticket secrets.

#### Scenario: resource access without output trust is blocked

GIVEN a client has a valid remote-build ticket or trusted-client authorization
AND the client lacks configured trust for the builder signing key, attestation authority, or required receipt policy
WHEN Mantle plans or completes a remote build
THEN Mantle MUST reject the remote route or output import as an output-trust blocker
AND it MUST NOT treat resource authorization as a cache hit, substitution hit, or successful remote build result.

#### Scenario: same-name different-key and stale output fail closed

GIVEN a returned output is signed by an unknown key, a same-name-but-different key, a revoked or expired key, or has stale object refs, wrong store prefix, missing attestation, or mismatched requested output identity
WHEN Mantle validates remote output admission
THEN Mantle MUST reject the output with deterministic diagnostics
AND it MUST NOT persist or export the tampered or untrusted output as successful.
