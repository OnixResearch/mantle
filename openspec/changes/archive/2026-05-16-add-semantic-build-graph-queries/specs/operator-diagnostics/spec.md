## ADDED Requirements

### Requirement: Operator diagnostics MUST expose semantic graph queries [r[operator-diagnostics.semantic-graph-queries]]

Mantle MUST expose operator-facing query commands for the semantic build graph. At minimum, Mantle MUST provide a graph view for a selected root, a why/explain view for one output or proof claim, and a dependents view for a selected identity. Human-readable and JSON output MUST expose the same core facts.

#### Scenario: Why query explains an output [r[operator-diagnostics.semantic-graph-queries.why]]

- GIVEN an output produced by a graph-capable build
- WHEN an operator runs `mantle why <output-or-identity>`
- THEN Mantle reports the producing recipe identity
- AND it reports relevant source, provider, sandbox, and proof receipt identities when present

#### Scenario: Incomplete graph is reported explicitly [r[operator-diagnostics.semantic-graph-queries.incomplete]]

- GIVEN an output that lacks required graph records
- WHEN an operator runs a graph query for that output
- THEN Mantle reports an incomplete-graph diagnostic
- AND it does not invent missing source, recipe, proof, or witness edges
