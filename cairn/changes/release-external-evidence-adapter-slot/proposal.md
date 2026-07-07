## Why

Mantle should remain a general release-evidence engine. The Valence/Octet/Trellis/Cairn provenance chain is valuable for our stack, but Mantle must not compile against or semantically depend on that stack to create or verify ordinary release bundles.

This change adds a generic, optional external-evidence sidecar slot to release evidence. Mantle verifies sidecar identity and bounded metadata. Stack-specific adapters, such as a Valence provenance adapter, own schema interpretation and stronger traceability claims outside Mantle core.

## What Changes

- Add an optional `external_evidence[]` collection to release evidence manifests.
- Each entry records role, schema, relative path, BLAKE3 digest, claim scope, and explicit non-claims.
- Release creation can copy one or more external evidence files into the bundle when requested by an operator or upstream adapter.
- Release verification checks path safety, file presence, digest match, role/schema/claim-scope presence, and non-claim presence.
- Add an opt-in verifier flag to require a named external-evidence role for stack-specific gates without changing default Mantle verification.
- Document that Mantle does not parse, validate, or promote Valence/Octet/Trellis/Cairn semantics; adapters own those checks.

## Impact

- **Mantle core remains stack-neutral**: no dependency on Valence, Octet, Trellis, or Cairn schemas beyond opaque sidecar metadata.
- **Our stack gets an integration seam**: Valence can produce a stack provenance sidecar that Mantle bundles and identifies.
- **Default compatibility is preserved**: release bundles without external evidence continue to verify under ordinary Mantle policy.
- **Non-claims**: external evidence identity does not prove the sidecar schema is semantically valid, complete, or sufficient for any stack-specific claim.
