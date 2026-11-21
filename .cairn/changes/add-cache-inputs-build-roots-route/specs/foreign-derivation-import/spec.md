# Foreign Derivation Import Cache-Inputs Build-Roots Delta

## ADDED Requirements

### Requirement: Cache-inputs build roots

r[foreign_derivation_import.cache_inputs_build_roots] Mantle MUST offer a
distinct preserved-path route `cache-inputs-build-roots-v1` that hydrates a
selected root's build inputs from the plan-bound signed Nix cache and executes
the selected root derivations without substituting them. The route MUST be
recorded in the existing executable-plan `realization_route` field and MUST be
selectable only when the translation policy preserves `/nix/store` output and
source paths exactly. Realization MUST require online substitution, one
receipt-bound cache URL with at least one trusted public key, and a
cache-closure policy, and MUST use the same narinfo discovery, signature
verification, bounded closure limits, exact-path-map admission and
post-import PathInfo re-verification as the cache-only route.

The hydrated seed set MUST contain exactly the outputs that the selected
roots' direct input-derivation edges request, the selected roots' own input
sources, and the runtime reference closure of those paths as declared by the
signed cache; Mantle MUST NOT walk the transitive derivation closure to collect
build-only dependency outputs. Every hydrated path MUST be inside the plan's
exact output/source path map, and a missing, unsigned, wrongly signed,
hash-mismatched, reference-incomplete or out-of-map input MUST fail closed
before any builder executes.

A selected root output present in the cache MUST be refused before store
mutation, and a selected root output already present in the local store MUST be
refused after hydration. A selected root whose builtin is a fixed-output fetch
MUST be refused at plan admission. The build store MUST be reopened without
substituters, the sandbox network MUST remain denied, and a declared source
input MUST NOT be supplied from the host logical store when it is absent from
the admitted store. Non-root units outside the hydrated closure MUST remain
`not-required-cache-only`; the scheduler MUST NOT build or hydrate them.

The realization receipt MUST keep the `mantle-foreign-realization-receipt-v1`
field set. Selected roots MUST report `execution_class: built`, hydrated
inputs MUST report `substituted` or `already-present`, the receipt MUST record
every hydrated closure fact, and the strongest state MUST remain `realized`
without claiming reproducibility, package correctness, evaluator equivalence,
provenance audit or proof admission.

#### Scenario: Root inputs hydrate and the root is built

GIVEN a preserve-cache plan selects one input-addressed root with `/nix/store`
paths
AND every direct input output and input source, and every runtime reference
the cache declares for them, is present with a trusted signed narinfo inside
the plan's exact path map
AND the cache does not offer any selected root output
WHEN real `cache-inputs-build-roots-v1` realization runs
THEN Mantle MUST hydrate and verify the input closure and record its facts
AND the selected root unit MUST execute in the sandbox and report `built`
AND every hydrated input unit MUST report `substituted` or `already-present`
AND the root output MUST be recorded from the realized PathInfo
AND the receipt MUST be `complete` with `strongest_state: realized`.

#### Scenario: A root offered by the cache is refused

GIVEN the plan-bound cache offers a narinfo for a selected root output, or the
root output is already present in the local store
WHEN realization starts
THEN Mantle MUST fail closed naming the selected root
AND it MUST NOT import input content, execute a builder, or publish a receipt.

#### Scenario: A missing or untrusted input stops the route

GIVEN a required input path has no narinfo, an untrusted signature, a
NAR-hash or path mismatch, a missing reference, an out-of-map member, an
exhausted closure limit, or exists only in the host logical store
WHEN hydration or build input resolution reaches it
THEN realization MUST fail closed with a stable bounded failure
AND no builder MAY execute as a fallback and no partial success MAY be
reported.

#### Scenario: The route is refused outside a preserved plan

GIVEN a plan with a recomputed target prefix or rewritten output/source paths
WHEN an operator requests `cache-inputs-build-roots-v1` at plan time or a plan
carrying the route is admitted
THEN Mantle MUST reject it before realization
AND a fixed-output selected root MUST be rejected with
`foreign-plan-cache-inputs-fod-root`.

#### Scenario: Unrelated units are not required

GIVEN a graph contains build-only units that are not requested by the selected
roots' direct input edges
WHEN the roots are realized
THEN Mantle MUST NOT hydrate or build those units
AND they MUST report `not-required-cache-only`.

## MODIFIED Requirements

### Requirement: Cache-only preserved foreign paths

r[foreign_derivation_import.cache_only_preserved_paths] Mantle MUST preserve exact foreign output paths only under an explicit cache-only policy with one unchanged logical store prefix. The route MUST remain distinct from executable local-build plans. Only the separate `cache-inputs-build-roots-v1` route may execute a root whose preserved paths are admitted; a plan may carry at most one such route, and execution authority MUST NOT be inferred from preserved path identity.

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
WHEN Mantle admits a plan under the cache-only route
THEN the route MUST remain cache-only and reject local build fallback
AND preserved path identity under that route MUST NOT authorize execution under either profile.
