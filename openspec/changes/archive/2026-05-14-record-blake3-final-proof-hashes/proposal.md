## Why

Crunch fixed-output compatibility hashes can remain SHA-256 SRI, but final proof metadata should always expose a Crunch-owned BLAKE3 proof hash so evidence has a stable preferred digest independent of fetch compatibility.

## What Changes

- Add BLAKE3 final proof metadata to the Clankers root bundle proof.
- Require external fixed bundle proofs to record a final BLAKE3 proof hash when SHA-256 compatibility hashes are also present.

## Impact

- Files: `packages/clankers/clankers-root-bundle.json`, `packages/clankers/clankers-root-proof.json`, `packages/clankers/clankers.ncl`, `openspec/specs/bootstrap/spec.md`.
- Verification: recompute the canonical proof hash and validate OpenSpec.
