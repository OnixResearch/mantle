## Why

Cairn can export evidence bundles and Mantle can bundle external release evidence, but the generic handoff between Cairn evidence export/import/index receipts and Mantle release evidence roles is not first-class. A contract would let Mantle carry Cairn lifecycle evidence without learning Cairn semantics.

## What Changes

- Add a Mantle release evidence role set for Cairn evidence exports.
- Validate Cairn bundle path, digest, role, schema, claim scope, and non-claims as opaque external evidence.
- Add positive and negative fixtures for complete, missing, stale, wrong-role, and weakened-boundary handoffs.

## Impact

- Cairn owns lifecycle/evidence semantics.
- Mantle owns release bundle linkage.
- Stack releases can include Cairn evidence without semantic coupling.
