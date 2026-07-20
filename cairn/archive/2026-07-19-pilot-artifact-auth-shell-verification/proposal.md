# Pilot exact artifact-auth shell verification for Mantle action results

## Why

Mantle's accepted artifact-auth adapter consumes an explicit cryptographic observation, but no Mantle shell currently creates that observation by signing and independently verifying the exact canonical `artifact-auth.statement.v1` bytes. Synthetic booleans and the existing detached action-result signature cover a different preimage and cannot support authority admission.

## What changes

- Expose a pure action-result-to-standalone-statement mapping from `crunch-action-result-core`.
- Add a thin `crunch-build` shell that requires an admitted legacy action-result decision, verifies the existing record signature and current key identity, signs exact standalone bytes with the existing Nix-compatible Ed25519 key adapter, and independently verifies through pinned `artifact-auth-ed25519`.
- Emit bounded statement, public-key, signature, legacy-authorization, currentness, failure-class, compatibility, and authority evidence.
- Add adversarial fixtures for tampering, wrong preimages, wrong keys, malformed carriers, currentness failure, authorization failure, drift, and false parity.
- Record a cross-consumer readiness comparison without admitting standalone authority.

## Impact

The action-result, PathInfo, OCI, repository, registry, credential, signing-key lifecycle, cache/build, receipt, and release gates remain authoritative. Standalone output stays diagnostic, rollback remains available, and no authority-admission change is created by this pilot.
