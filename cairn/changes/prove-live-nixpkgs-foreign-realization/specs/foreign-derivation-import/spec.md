## ADDED Requirements

### Requirement: Cache-only preserved foreign paths

r[foreign_derivation_import.cache_only_preserved_paths] Mantle MUST preserve exact foreign output paths only under an explicit cache-only policy with one unchanged logical store prefix. The route MUST remain distinct from executable local-build plans.

#### Scenario: Exact Nix cache paths are admitted

GIVEN a concrete Nix graph uses one `/nix/store` source prefix
AND policy selects `/nix/store` with `preserve-cache-paths-v1`
WHEN Mantle compiles the executable plan
THEN every foreign output and source path MUST map to itself exactly
AND cache-only route identity, execution profile, graph, policy, and package selection MUST remain bound into plan identity.

#### Scenario: Changed prefix fails closed

GIVEN policy requests preserved cache paths with a changed or ambiguous prefix
WHEN Mantle validates or compiles the graph
THEN it MUST reject the policy before realization
AND it MUST NOT emit a partial executable plan.

#### Scenario: Cache-only path cannot authorize execution

GIVEN one preserved output path is referenced under different builder or profile facts
WHEN Mantle admits the plan
THEN the route MUST remain cache-only and reject local build fallback
AND preserved path identity MUST NOT authorize execution under either profile.

### Requirement: Signed cache-only runtime closure

r[foreign_derivation_import.cache_only_runtime_closure] Mantle MUST hydrate a selected preserved-path runtime closure through its signed PathInfo, NAR, castore, scheduler, worker, and store boundaries. A miss or invalid fact MUST NOT fall back to a local foreign build.

#### Scenario: Selected closure is hydrated

GIVEN a selected root and every runtime reference have trusted signed cache facts
WHEN cache-only realization runs
THEN Mantle MUST verify and ingest each bounded closure member through normal store admission
AND the normal scheduler MUST observe the selected root as present without executing a builder.

#### Scenario: Invalid closure member stops the route

GIVEN one required cache member is missing, unsigned, wrongly signed, NAR-mismatched, path-mismatched, or incomplete
WHEN hydration reaches that member
THEN realization MUST stop with a stable bounded failure
AND no local or remote builder MAY execute as fallback.

#### Scenario: Closure limits fail closed

GIVEN runtime closure hydration exceeds its path, reference, byte, or depth limit
WHEN the limit is reached
THEN Mantle MUST stop with a stable limit failure
AND it MUST NOT report complete realization.

#### Scenario: Builder sources are not consumed

GIVEN a cache-only plan includes derivation-time input source declarations
WHEN the selected runtime closure is already available from trusted cache facts
THEN the source bundle MUST be empty and no builder source MAY be materialized
AND units outside the selected runtime closure MUST be reported as not required.

### Requirement: Live Nixpkgs realization proof

r[foreign_derivation_import.live_nixpkgs_realization_proof] Mantle MUST retain reproducible evidence for one live host-Nix `nixpkgs#hello` export that reaches receipt-bound realized and provenance-audited states without a Nix frontend during consumption.

#### Scenario: Live export reaches realized state

GIVEN host Nix exports a concrete recursive `nixpkgs#hello` derivation graph
WHEN Mantle produces, validates, plans, and consumes the artifacts with Nix commands absent from PATH
THEN the selected signed runtime closure MUST reach `realized` through cache-only Mantle realization
AND the evidence MUST bind graph, policy, plan, build report, realization receipt, cache closure, and output identities.

#### Scenario: Reuse and provenance are checked

GIVEN the live closure completed once
WHEN realization and provenance audit run again against the same state
THEN exact output reuse MUST avoid builder execution and the audit MUST report its bounded disposition
AND evidence MUST preserve any failed findings rather than promote an unsupported claim.

#### Scenario: Proof keeps explicit non-claims

GIVEN live `nixpkgs#hello` evidence is complete
WHEN an operator reviews the evidence
THEN local rebuild compatibility, evaluator parity, package correctness, reproducibility, bootstrap parity, runtime safety, and release eligibility MUST remain non-claims
AND future Nixpkgs revisions and cache availability MUST remain outside the proof.
