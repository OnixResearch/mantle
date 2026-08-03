# Structure Mantlepkgs package domains

## Why

The active Mantlepkgs change defines deterministic catalog generation from locked concrete graphs. It does not define package domains, explicit variants, shard composition, or separate validation roots.

A single undifferentiated catalog makes ownership and review harder. Tests inside package builds also make test-only changes alter package output identities.

## What Changes

- Add typed `core` and named `ecosystem` package domains.
- Compose domain shards through one deterministic catalog index.
- Model package variants as explicit records instead of flattened selector conventions.
- Model package validation as separate roots that consume package outputs.
- Use one pinned Ekala `corepkgs` revision as an external validation corpus.
- Keep composition, variant, and validation-root decisions in a pure functional core.

## Non-Goals

- Importing Ekala Nix expressions into Mantle runtime code.
- Adding Nix evaluation to Mantlepkgs consumers.
- Treating domain class as package trust, quality, or release authority.
- Adding overlay semantics or implicit package replacement.
- Claiming broad Ekala or Nixpkgs compatibility from one corpus.

## Dependencies

- `generate-mantlepkgs-from-nixpkgs` supplies the initial catalog producer and consumer boundary.
- ADR 0056 defines concrete-graph generation and the no-Nix consumer boundary.

## Impact

- **Affected specs:** new `mantlepkgs-catalog-structure` specification.
- **Affected code:** Mantlepkgs Nickel contracts, catalog core, CLI shell, generated indexes, and validation-root adapters.
- **Affected evidence:** package-domain receipts, corpus provenance, positive fixtures, negative fixtures, and no-Nix consumer evidence.
- **Compatibility:** existing single-domain catalogs need an explicit migration or a versioned compatibility decoder.
