## ADDED Requirements

### Requirement: StageX lineage provider materialization

r[bootstrap_inventory.stagex_lineage_provider_materialization] Mantle MUST materialize a bounded intermediate StageX provider from an audited hex0 seed through the declared protected TinyCC, native-musl, and binutils transition, with a real BLAKE3-bound receipt, before marking `seed-full.stagex-lineage` complete.

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

#### Scenario: explicit inputs control atomic publication

GIVEN an absolute lineage manifest, an absolute complete transition root, and an absent absolute output path
WHEN Mantle materializes the provider
THEN Mantle MUST publish target-prefixed tools, headers, libraries, provider metadata, validation evidence, and the receipt with create-new staging and Linux no-replace rename
AND relative paths, inferred transition roots, an existing destination, symlinks, partial staging, or a destination collision MUST fail closed.

#### Scenario: complete receipt replaces scaffold evidence

GIVEN every declared stage succeeds and the normalized provider passes independent contract and relocated runtime validation
WHEN Mantle publishes and revalidates the provider and StageX receipt
THEN the receipt MUST contain audited-seed, lineage-manifest, stage-graph, source-state, normalized-provider, output, transition-report, protected-audit, provider-validation-audit, provider-validation-report, receipt-payload, and final-bundle BLAKE3 identities with `lineage_receipt_status = complete` and no fallback events
AND live provider components MUST use observed file or tree identities
AND every stage authorization MUST bind to an observed protected-audit `execve` or `execveat` decision with the same absolute path and BLAKE3 digest identity
AND exact allowlisted report-only observations MUST bind the artifact ID, plan digest, and complete transition-report digest
AND partial output, scaffold values, stale evidence, or validation failure MUST leave `seed-full.stagex-lineage` blocked.

#### Scenario: StageX claim remains bounded

GIVEN the StageX provider row is complete
WHEN the result is cited
THEN Mantle MUST identify the seed, graph, source state, environmental assumptions, transition, protected-exec audit, provider, and runtime evidence
AND it MUST NOT claim seed correctness, compiler correctness, complete musl or binutils behavior, final native GCC provider admission, kernel isolation, independent rebuild agreement, release reproducibility, or Mantle self-build completion.