## Why

Foreign derivation import receipts are intentionally not build-success, correctness, or reproducibility proofs. Without operator-facing trust-model documentation, the new import boundary could be misread as a stronger claim than it is.

Mantle needs clear docs and guards explaining what a foreign import receipt means, what it does not mean, and what additional evidence is required before trusting imported outputs.

## What Changes

- Add operator documentation for the foreign derivation import trust model.
- Explain graph provenance, policy digests, source/cache trust scopes, sandbox capabilities, and receipt non-claims.
- Provide examples for Guix-like and Nix-like imports that distinguish import admission from realization and verification.
- Add a doc guard that fails when trust/non-claim sections disappear from operator docs.

## Impact

- **Files**: docs, README links, guard script/tests.
- **Testing**: doc guard positive/negative tests and Cairn gates.

## Out of Scope

- Implementing a live Guix or Nix graph generator.
- Claiming imported outputs are trusted solely because a receipt exists.
- Changing the foreign import core schema unless docs expose a gap.
