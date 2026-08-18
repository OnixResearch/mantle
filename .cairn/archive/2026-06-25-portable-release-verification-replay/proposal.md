# portable release verification replay

## Problem

The deterministic release proof and provider fixed-point proof were verified from the Mantle checkout using local generated paths. That proves the verifier can evaluate the current artifacts on this host, but it does not yet prove the evidence set can be copied to a clean location and verified without relying on incidental source checkout state, current working directory assumptions, or stale `target/` siblings.

Operators need a portable verification replay rail for release evidence: given the release bundle plus required proof sidecars, a verifier should be able to run from a fresh scratch directory and reach the same bounded result.

## Proposed change

Add a Cairn-scoped verification rail that exports or copies the minimal release verification artifact set to a fresh scratch directory and reruns `mantle release verify` from outside the source checkout with both deterministic-release and provider fixed-point proof required.

The rail should also include a negative replay that removes or withholds required deterministic proof evidence and confirms verification fails closed rather than silently weakening the claim.

## Success criteria

- The replay uses a fresh scratch/export directory and does not depend on `target/` sibling discovery.
- Required verification succeeds from copied release bundle and proof sidecars.
- A negative replay with missing deterministic proof material fails closed with a deterministic diagnostic.
- Evidence records exact commands, paths, verifier JSON status, and bounded non-claims.
- Cairn validation remains clean after evidence is recorded.
