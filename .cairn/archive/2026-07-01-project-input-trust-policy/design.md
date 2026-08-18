# Design: Project input trust policy

## Architecture

Trust verification is a shell operation that produces pure trust facts for lock planning.

- Pure core: trust policy schema validation, verified-fact normalization, required-signer/quorum decisions, lock update acceptance/rejection, project attestation data shaping, and bounded non-claim wording.
- Imperative shell: loading signature/public-key material, running minisign/PGP or supported verifier implementations, reading fetched bytes, and rendering diagnostics.

The core must not read key files, contact key servers, or run verifier processes. It should decide only from explicit `VerifiedInputTrust` records.

## Policy model

Trust policy should be attachable to inputs and patch definitions. A policy may require detached or inline signature refs, public verifier tokens, signer names or key fingerprints, key-set quorums when supported, and binding to the fetched bytes' BLAKE3 or required protocol digest. Missing trust policy means hash-only integrity, not trusted signature evidence.

Trust results should be recorded in refresh reports and optionally projected into project attestations as bounded claims: bytes matched the configured trust policy at refresh time.

## Failure behavior

When a policy requires trust evidence, lock update must fail before writing new source hashes if signature material is missing, malformed, invalid, from an untrusted key, detached from the fetched bytes, or verified by an unsupported algorithm. Existing lock entries should remain untouched on trust failure.

## Validation strategy

Pure positive tests should cover accepted signer facts, quorum satisfaction, input and patch policy application, lock update acceptance, and attestation claim shaping. Pure negative tests should cover missing signatures, wrong keys, bad signatures, digest mismatch, unsupported verifier kinds, malformed key refs, and overbroad trust claims.

Shell tests should use local fixture keys/signatures and prove no key-server or forge trust is used implicitly.
