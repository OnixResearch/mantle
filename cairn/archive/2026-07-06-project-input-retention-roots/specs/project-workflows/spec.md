## ADDED Requirements

### Requirement: Retention root offline proof rail emits bounded versioned evidence

r[project_workflows.retention_root_proof_rail] Mantle MUST provide a bounded local offline proof rail that exercises `untracked`, `current`, and `recent-generations` input retention through refresh, import, and generated-input updates and `mantle check`, commits retention roots atomically into `.mantle/retention.json` and `.mantle/retention-roots/`, and emits a versioned, redacted, non-overclaiming evidence record that classifies each input as pinned, unpinned, stale-root, missing-root, or garbage-collection-eligible, binds the input name, lock digest, source identity, and content digest, and validates named bounded generation limits before roots are treated as durable.

#### Scenario: current input is pinned after its root exists

GIVEN a project input has retention set to track the current locked source
WHEN Mantle commits a refresh, import, or generated-input update for that input through the rail
THEN Mantle MUST plan or create a retention root bound to the input name, lock digest, source identity, and content digest
AND diagnostics MUST report the input as pinned only after the root exists.

#### Scenario: recent generations retain bounded lock-fact-based history

GIVEN a project default or input override retains recent generations
WHEN Mantle updates the lockfile across multiple generations through the rail
THEN Mantle MUST retain no more than the configured bounded generation limit for that input
AND generation selection MUST be based on Mantle-owned lock generation facts rather than filesystem timestamp ordering.

#### Scenario: interrupted stale and untracked roots are diagnosed

GIVEN a retention root update is interrupted, or a root is stale, missing, or untracked
WHEN Mantle checks project soundness through the rail
THEN Mantle MUST treat uncommitted retention records as absent or quarantined, report stale-root or missing-root diagnostics, and report an untracked input as garbage-collection-eligible
AND it MUST NOT count a stale, missing, or untracked root as satisfying current retention or as durable.

#### Scenario: evidence is versioned redacted and non-overclaiming

GIVEN the rail emits its evidence record
WHEN the record is rendered
THEN it MUST carry a stable schema version, per-input mode and classification, bound input name/lock digest/source identity/content digest, generation limit when applicable, the atomicity assertion, and non-claims
AND it MUST omit raw environment values, private key material, and unbounded logs and MUST NOT claim build correctness or release reproducibility from retention roots.
