## ADDED Requirements

### Requirement: Source-built Mantle fixed-point proof

r[bootstrap_inventory.source_built_mantle_fixed_point] Mantle MUST claim a full-source fixed point only when one authenticated proof authority constructs or revalidates the complete StageX native provider and full-source-bound Rust provider, builds stage1 Mantle, and uses stage1 Mantle to build a BLAKE3-identical stage2 through the same Cargo-free, strict-hermeticity, no-fetch, and no-fallback policy.

#### Scenario: proof inputs are sources rather than provider outputs

GIVEN a fresh proof root and independently authenticated source bundle
WHEN the full-bootstrap proof begins
THEN it MUST validate the audited seed, lineage manifest, native and Rust source records, vendor inputs, source-state identity, policy identities, resource bounds, and empty native-provider, Rust-provider, and Mantle output authorities before construction
AND an imported or cache-hit native provider, Rust provider, Mantle binary, missing source, wrong digest, stale state, or producer-checkout path authority MUST fail preflight.

#### Scenario: root action trust is complete before execution

GIVEN the proof inputs, policies, and six-stage authority plan are valid
WHEN Mantle admits the proof for execution
THEN it MUST emit a deterministic root-scoped action trust plan that enumerates every reachable action, broad stage, producer edge, fixed or produced executable authority, input authority, output, local-only execution rule, event-count bound, and resource limit
AND a generated path without a producer action and output identity, an incomplete action adapter, an unknown input authority, remote execution, or cache-only completion MUST fail before construction.

#### Scenario: stage1 uses the constructed closure

GIVEN the StageX native provider and full-source-bound Rust provider were constructed from source inside the current proof authority
WHEN stage1 Mantle is planned and built
THEN every planner, rustc, build script, proc macro, native compiler/linker, runtime, and source read MUST be authorized by the immutable receipt-bound closure and source policy
AND Cargo invocation, live fetch, ambient executable/read discovery, practical hermeticity fallback, or protected-exec violation MUST fail stage1.

#### Scenario: stage1 builds stage2

GIVEN stage1 Mantle builds and smokes successfully
WHEN stage2 starts
THEN the produced stage1 Mantle binary MUST plan and execute stage2 using the same source-state, provider, closure-policy, Cargo-free, hermeticity, and protected-execution identities
AND the host Mantle binary, a different provider/closure, or changed authority plan MUST NOT substitute for stage1.

#### Scenario: planned and observed actions reconcile

GIVEN the complete root action trust plan and stage execution records
WHEN Mantle evaluates execution authority
THEN every protected-exec and build execution record MUST map to one planned action, permitted executable authority, producer relationship, BLAKE3 identity, locality rule, and event-count bound
AND an unknown event, missing required event, digest drift, producer drift, count-bound violation, remote execution, or undeclared input MUST preserve a failed proof.

#### Scenario: fixed point emits v2 evidence

GIVEN both stages complete
WHEN Mantle evaluates fixed-point success
THEN stage1 and stage2 Mantle binary BLAKE3 digests MUST match and a verified `mantle-deterministic-proof-receipt-v2` MUST bind source/rebuild descriptors, authority plan, provider/closure identities, stage receipts, action-trust plan, observed execution reconciliation, approved reads, effects, audits, outputs, and final proof-bundle digest
AND any mismatch, malformed receipt, fallback, unapproved effect, action-trust violation, or incomplete evidence MUST preserve a failed proof without updating successful aliases.

#### Scenario: fixed-point claim remains bounded

GIVEN the full-source fixed point succeeds
WHEN operators or release tooling cite it
THEN the claim MUST identify seed, source, lineage, native/Rust providers, closure policy, orchestration authority, root action trust plan, planned-versus-observed execution result, platform, stage outputs, audits, and matching digests
AND it MUST NOT claim compiler correctness, seed correctness, kernel isolation, independent rebuild agreement, bit-for-bit release reproducibility, deployment success, or full Cargo compatibility.