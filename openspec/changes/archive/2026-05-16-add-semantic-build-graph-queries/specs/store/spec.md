## ADDED Requirements

### Requirement: Store records SHOULD link PathInfo identities to semantic graph nodes [r[store.semantic-graph-links]]

Mantle store state SHOULD link persisted PathInfo/output identities to semantic graph nodes when graph records are available. Store APIs MUST tolerate missing graph links for legacy entries and return a typed missing-link result rather than failing unrelated store operations.

#### Scenario: Store lookup returns graph link when present [r[store.semantic-graph-links.present]]

- GIVEN a PathInfo entry created by a graph-capable build
- WHEN a caller requests its graph link
- THEN the store returns the corresponding output node identity

#### Scenario: Legacy store entry has no graph link [r[store.semantic-graph-links.legacy]]

- GIVEN a legacy PathInfo entry without graph metadata
- WHEN a caller requests its graph link
- THEN the store returns a typed missing-link result
- AND ordinary cache/build reuse remains unaffected
