## Why

The smoke path needed custom code to bind Valence/Kamacite function-address evidence into a Mantle release context and produce a Cairn-consumable binding. Mantle owns release artifact binding, so this should be a first-class release command rather than ad hoc integration glue.

## What Changes

- Add a Mantle CLI surface to bind function-address sidecars/receipts to a release manifest or release evidence bundle.
- Validate sidecar role, schema, claim scope, source archive digest, release binary digest, optional Kamacite receipt metadata, and non-claims.
- Emit a deterministic binding receipt for Cairn readiness.
- Add positive and negative fixtures for optional and required policy modes.

## Impact

Mantle can prove bundle-local function-address evidence linkage without parsing functions or claiming semantic correctness.
