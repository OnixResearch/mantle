## ADDED Requirements

### Requirement: Prefix-aware concrete ATerm producer

r[foreign_derivation_import.prefix_aware_aterm] Mantle MUST parse explicit concrete ATerm derivation bundles under one declared foreign store prefix. Parsing MUST remain independent of Guix, Nix, foreign evaluators, and host store discovery.

#### Scenario: Guix derivation bundle parses without Guix

GIVEN an explicit complete `.drv` bundle uses the declared `/gnu/store` prefix
WHEN the producer parses the bundle into `foreign-derivation-graph-v1`
THEN it MUST preserve derivations, outputs, input edges, sources, builders, arguments, environments, fixed-output facts, builtins, and references
AND it MUST NOT execute `guix`, a Scheme evaluator, or a foreign store command.

#### Scenario: Nix derivation bundle preserves compatibility

GIVEN an explicit complete `.drv` bundle uses the declared `/nix/store` prefix
WHEN the prefix-aware producer parses the bundle
THEN it MUST emit the same canonical graph facts as the compatible Nix producer
AND it MUST NOT replace required Nix-format hash facts with Mantle BLAKE3 values.

#### Scenario: Mixed or malformed prefix fails closed

GIVEN a derivation bundle contains malformed store paths or paths outside its declared source prefix
WHEN the producer validates the bundle
THEN it MUST reject the bundle with a deterministic path diagnostic
AND it MUST NOT emit a partial graph or package index.

### Requirement: Exact dependency-ordered graph compilation

r[foreign_derivation_import.exact_graph_compilation] Mantle MUST compile accepted foreign graph nodes in deterministic dependency order. Every foreign derivation, output, and source store object MUST map to its exact recomputed target object before a parent can use it.

#### Scenario: Parent receives exact child output

GIVEN a child node produces one target output and a parent references the foreign child output
WHEN Mantle compiles the child and then the parent
THEN the path used by the parent MUST equal the child’s recomputed target output path byte for byte
AND the executable plan MUST record both the foreign and target identities.

#### Scenario: Path suffix is preserved

GIVEN a builder field references a declared foreign output followed by `/bin/tool`
WHEN the compiler rewrites that field
THEN it MUST replace only the mapped store object and preserve `/bin/tool`
AND it MUST reject the field if the base store object has no exact mapping.

#### Scenario: Cycle or incomplete order fails closed

GIVEN the reachable graph contains a cycle, missing node, duplicate edge, unknown output, or omitted reachable node
WHEN dependency ordering completes
THEN compilation MUST fail with a stable graph diagnostic
AND no successful executable plan MUST be emitted.

#### Scenario: Leftover foreign reference fails closed

GIVEN a rewritten builder, argument, environment, source, or reference field still contains a declared foreign store object
WHEN final graph validation runs
THEN compilation MUST fail with an undeclared or unmapped reference diagnostic
AND it MUST NOT approximate the target path by replacing only the prefix.

### Requirement: Bounded foreign builtin lowering

r[foreign_derivation_import.foreign_builtin_lowering] Mantle MUST lower only explicitly supported foreign builtin operations. The first compatibility set MUST cover declared download and Git download forms with fixed-output verification facts and deterministic source-candidate order.

#### Scenario: Download builtin becomes Mantle fetch facts

GIVEN a foreign fixed-output node declares a supported download builtin, content hash, mode, executable flag, and ordered candidates
WHEN graph compilation lowers the node
THEN the resulting native unit MUST use Mantle fetch facts with the same verified content requirement
AND source candidate order MUST remain deterministic.

#### Scenario: Git download preserves revision facts

GIVEN a supported Git download node declares a repository, revision, recursive hash, and export policy
WHEN the compiler lowers the node
THEN the native unit MUST bind those facts without invoking a foreign fetcher
AND Mantle’s later fetch service MUST remain responsible for acquisition and verification.

#### Scenario: Unsupported builtin is rejected

GIVEN a node names a builtin outside the accepted mapping policy
WHEN graph compilation reaches that node
THEN it MUST fail with a deterministic unsupported-builtin diagnostic
AND it MUST NOT invoke an external helper or approximate the operation.

#### Scenario: Hash domains remain distinct

GIVEN a source-format SHA-256 value and Mantle-owned BLAKE3 plan identity exist for one node
WHEN the executable plan is emitted
THEN each digest MUST declare its algorithm and role
AND either digest supplied in the other role MUST be rejected.

### Requirement: Receipt-bound executable foreign plan

r[foreign_derivation_import.executable_plan] Mantle MUST emit a deterministic `mantle-foreign-executable-plan-v1` before foreign realization. The plan MUST bind accepted import identity, target roots, native units, exact path maps, source requirements, execution-profile references, diagnostics, and explicit non-claims.

#### Scenario: Plan is reviewable without a foreign frontend

GIVEN an accepted foreign graph and translation policy compile successfully
WHEN an operator inspects the executable plan
THEN the plan MUST show each selected root, target unit, path mapping, source requirement, and policy identity
AND inspection MUST NOT require Guix, Nix, flakes, overlays, or package-module evaluation.

#### Scenario: Plan does not claim realization

GIVEN a valid executable plan exists
WHEN Mantle reports its strongest proven state
THEN the state MUST remain compilation or planning only
AND source availability, scheduler execution, store admission, output trust, package correctness, and reproducibility MUST remain non-claims.

#### Scenario: Partial root compilation emits no success

GIVEN one selected root compiles and a sibling selected root fails
WHEN the compiler prepares the executable plan
THEN it MUST report bounded diagnostics for the failed selection
AND it MUST NOT emit a successful plan that omits the failed selected root.
