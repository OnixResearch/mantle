# Design: provider fixed-point candidates for witness rebuild

## Boundary

`mantle release witness-rebuild` remains the imperative shell for witness-side release replay. It still extracts the source archive, runs the supported workflow driver, collects rebuilt artifacts, validates digests, and only then creates witness sidecars.

The functional core change is limited to rebuilt-candidate discovery:

1. load self-hosting proof candidates from the workflow proof bundle as today;
2. if the workflow wrote a provider fixed-point proof bundle under the witness scratch root, validate that bundle with the existing provider fixed-point verifier;
3. resolve the provider stage binaries from the provider proof's `meta.json`;
4. require resolved provider stage binaries to stay inside the witness-owned provider proof directory;
5. append those verified stage binaries to the digest-indexed candidate set.

## Workflow handoff

The Rust witness shell exports two additional environment variables to the workflow driver:

- `CRUNCH_WITNESS_REBUILD_REPO_DIR`: extracted source tree path;
- `CRUNCH_WITNESS_PROVIDER_FIXED_POINT_PROOF_BUNDLE_DIR`: witness-owned directory where a driver may write a fresh provider fixed-point proof.

The default self-hosting driver can ignore these variables. Operator wrappers can run the provider fixed-point proof before or after the self-hosting proof and write it to the exported directory.

## Safety properties

- Candidate selection remains by BLAKE3 digest, not output position or filename.
- Provider fixed-point proof validity is checked before its stage binaries are eligible.
- Provider stage paths may be absolute because the fixed-point proof format records absolute paths, but they must canonicalize under the witness-owned provider proof directory.
- The release bundle's bundled publisher provider proof is never consulted as a rebuilt output source.
- If the provider proof is missing or invalid, the existing missing-digest failure path prevents witness sidecar creation.
