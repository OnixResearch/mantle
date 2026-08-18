## Why

Mantle's first landed artifact-auth operational canary passed against a real non-production build action result, but its public receipts and exact harness remain only in an unversioned workspace directory. Losing or partially copying that evidence would prevent later reviewers from distinguishing the observed result from a reconstruction or an authority claim.

## What Changes

- Archive the Mantle-specific public artifacts from `artifact-auth/run-001` with the exact Mantle and artifact-auth revisions.
- Preserve the exact temporary harness, real build report, operational receipt, successful fresh-process replay, revocation transition, and fresh-process denial.
- Add a typed Nickel manifest and reproducible BLAKE3 inventory while excluding private signing material and secret state.
- Keep the canary explicitly non-production and non-authoritative; legacy behavior remains authoritative and rollback remains available.

## Impact

- **Files**: this Cairn change, its evidence bundle, and the accepted `artifact-auth-operational-receipt` specification.
- **Testing**: Nickel typecheck, deterministic BLAKE3 regeneration, negative symlink rejection, secret scan, expected-denial inspection, Cairn gates, sync, and archive validation.
