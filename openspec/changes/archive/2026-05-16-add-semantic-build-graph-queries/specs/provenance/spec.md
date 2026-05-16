## ADDED Requirements

### Requirement: Mantle MUST persist a queryable semantic build graph [r[provenance.semantic-build-graph]]

Mantle MUST persist a semantic build graph for new graph-capable builds and proof operations. The graph MUST represent immutable node identities for source trees, recipes, store outputs, sandbox profiles, provider identities, proof receipts, witness requests, and release evidence. Human-readable names, aliases, package labels, and legacy names MUST be represented as metadata edges or alias records, not as the identity of graph nodes.

#### Scenario: Name changes do not change content identity [r[provenance.semantic-build-graph.names-metadata]]

- GIVEN a recipe node with a stable content identity
- WHEN a human-readable alias for that recipe changes
- THEN the recipe node identity remains unchanged
- AND the graph records the alias update separately from the recipe identity

#### Scenario: Output graph links proof material [r[provenance.semantic-build-graph.proof-links]]

- GIVEN a release output with a deterministic proof receipt and witness request
- WHEN Mantle persists graph records for the release
- THEN the output node links to the producing recipe, sandbox profile, proof receipt, release evidence, and witness request nodes
- AND each edge kind is machine-readable
