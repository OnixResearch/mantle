## ADDED Requirements

### Requirement: StageX lineage provider materialization

r[bootstrap_inventory.stagex_lineage_provider_materialization] Mantle MUST materialize the StageX lineage provider from an audited hex0 seed through declared transition and native-toolchain stages, with a one-way protected execution transition and a real BLAKE3-bound receipt, before marking `seed-full.stagex-lineage` complete.

#### Scenario: lineage plan binds the real root and graph

GIVEN an audited hex0 seed, authenticated source state, and declared environmental assumptions
WHEN Mantle plans StageX materialization
THEN the plan MUST bind seed bytes and digest, stage graph, immediate predecessor edges, source identities, expected output roles, execution limits, protected-transition point, and environmental assumptions
AND missing, duplicate, cyclic, substituted, unbounded, or digest-mismatched stages MUST fail before execution.

#### Scenario: protected transition forbids fallback

GIVEN Mantle has verified the seed/source authority and reached the declared protected transition
WHEN later lineage stages execute
THEN every executable MUST be authorized by exact absolute path, role, source stage, and BLAKE3 digest and every decision MUST be retained in the protected-exec audit
AND host shell, BusyBox, bwrap, compiler/linker, checkout discovery, live source acquisition, relative executable, digest mismatch, or any fallback event MUST fail the lineage.

#### Scenario: environmental assumptions stay outside produced lineage

GIVEN the host kernel, initial Mantle orchestrator, or pre-transition launcher remains an execution assumption
WHEN Mantle emits the lineage receipt
THEN each assumption MUST be named, bounded, trust-classified, and separated from seed and produced-stage identities
AND assumption bytes or effects MUST NOT be relabeled as source-built provider outputs.

#### Scenario: complete receipt replaces scaffold evidence

GIVEN every declared stage succeeds and the normalized provider passes independent contract and runtime validation
WHEN Mantle publishes the provider and StageX receipt
THEN the receipt MUST contain observed audited-seed, lineage-manifest, stage-graph, source-state, normalized-provider, output, protected-audit, and final-bundle BLAKE3 identities with `lineage_receipt_status = complete` and no fallback events
AND partial output, scaffold values, stale evidence, or validation failure MUST leave `seed-full.stagex-lineage` blocked.

#### Scenario: StageX claim remains bounded

GIVEN the StageX provider row is complete
WHEN the result is cited
THEN Mantle MUST identify the seed, graph, source state, environmental assumptions, transition, protected-exec audit, provider, and runtime evidence
AND it MUST NOT claim seed correctness, compiler correctness, kernel isolation, independent rebuild agreement, release reproducibility, or Mantle self-build completion.