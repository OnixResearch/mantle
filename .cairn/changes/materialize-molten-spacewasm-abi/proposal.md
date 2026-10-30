# Add consumer-owned Molten ABI fixtures to SpaceWasm bundles

## Why

The archived reference cohort materializes eight hostless checks. They cannot
stand in for a consumer's hostcall, canonical Preserves, memory or suspension
contract. Molten's pilots need exact producer-owned modules without building
new guests inside the consumer repository.

## What changes

Retain the reviewed NASA revision and reference profile. Add generated
core-Wasm fixture members for host segmentation and `molten.wasm.abi.v1`, with
WAT authoring sources, descriptors, exact BLAKE3 member identities, binary
readback, and missing/mismatch/substitution negative controls. Consumer receipts
remain separate; no new reference-runner outcome is asserted.

## Impact

- `nix/spacewasm-reference.nix` imports finite additive member producers.
- `packages/spacewasm-reference` retains the segmentation authoring source.
- Existing public package and verifier names remain unchanged.
- Molten, Octet and ChaosControl bind fixture identities and their own policy;
  publication authority pins the final immutable Mantle revision and bundle.

## Non-goals

No production backend switch, runtime correctness claim, new upstream version,
hand-edited generated binary or hash, or consumer-side guest build.
