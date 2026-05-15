## Why

Mantle now has bounded release-artifact reproducibility reporting, but that is
not the same as a Nix-like deterministic build claim. Operators need a separate
contract for when Mantle may say a derivation or release artifact is
"deterministically reproducible": the evidence must include repeated clean
builds, ambient host perturbation, canonical input identity, strict hermetic
execution, and BLAKE3 equality of canonical outputs.

## What Changes

- Define a deterministic-build proof as a stronger evidence class than a single
  successful release rebuild.
- Require a canonical determinism proof receipt that records input identity,
  workflow/toolchain identity, hermeticity mode, ambient perturbations, audit
  events, BLAKE3 output digests, and verdict.
- Require repeated clean builds in isolated stores before a deterministic claim
  is accepted.
- Keep claims bounded: a proven derivation/release does not imply global system
  determinism.

## Capabilities

### New Capabilities
- `build-pipeline.determinism.proof`: per-derivation deterministic proof
  receipts and verification rules.
- `release-verification-tech.determinism.claims`: release verification can
  consume deterministic proof receipts without overclaiming full-system
  determinism.

### Modified Capabilities
- `build-pipeline.hermeticity`: strict hermetic mode becomes the prerequisite
  for deterministic proof attempts.

## Impact

- **Files**: build pipeline, release verification, CLI docs/tests, proof receipt
  schema.
- **APIs**: likely adds a proof command or flag in implementation, but this
  OpenSpec defines the contract first.
- **Testing**: regression harness must run repeated clean builds under perturbed
  ambient host state and compare BLAKE3 NAR/output digests.
