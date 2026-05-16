## Why

Unison gains leverage by storing processed code in a codebase database with stable identities, dependencies, and type-aware query surfaces. Mantle has analogous data spread across PathInfo, provenance, release evidence, parity receipts, and build logs. Operators need one semantic graph they can query when asking why an output exists, which proof supports it, which source/recipe/sandbox facts contributed, or which dependents would be affected by a change.

## What Changes

- **Semantic graph model**: Define first-class node and edge kinds for source trees, recipes, store outputs, sandbox profiles, providers, proof receipts, witness requests, and names/aliases.
- **Query surfaces**: Add operator queries such as `mantle graph`, `mantle why`, `mantle dependents`, and proof explanation output.
- **Name metadata**: Treat human names and aliases as metadata over immutable BLAKE3-backed identities.

## Capabilities

### New Capabilities
- `semantic-build-graph`: inspectable build/proof graph.

### Modified Capabilities
- `provenance`: graph nodes/edges become canonical query material.
- `operator-diagnostics`: operator-facing graph and why/explain commands.
- `store`: store identities can link to graph records.

## Impact

- **Files**: provenance model, local state schema, CLI diagnostics, tests, docs.
- **APIs**: additive graph query APIs and CLI surfaces.
- **Testing**: graph round-trip tests, query output snapshots, negative tests for missing graph edges.
