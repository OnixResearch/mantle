## ADDED Requirements

### Requirement: Pipeline layer maintains strict Tiger Style conformance

r[build_correctness.pipeline_tiger_conformance] Mantle MUST keep `crunch-pipeline` within the complete pinned Tiger Style policy without lint allowances, warning budgets, finding baselines, target-scope reductions, or weaker full-check enforcement, while preserving eager evaluation completeness, registered-build ordering, managed-root authority, cache-only trust, evidence collection, failure normalization, and public compatibility.

#### Scenario: strict pipeline check accepts the package

GIVEN pipeline source uses admitted collection bounds, checked capacity arithmetic, coherent private interfaces, and capability-aligned functions
WHEN the focused package check and repository Tiger Style check run
THEN both MUST report zero `crunch-pipeline` findings without an allowance or suppressed target
AND positive and negative package tests, strict Clippy, formatting, and caller compilation MUST pass.

#### Scenario: eager evaluation exceeds its admitted root count

GIVEN the evaluation session declares a bounded root count
WHEN streamed messages would add more roots than that count
THEN Mantle MUST reject the stream through a typed error before collection grows beyond the bound
AND ordinary complete streams MUST retain every root and discovered action.

#### Scenario: managed-generation planning changes root authority

GIVEN outputs and source paths produce a deduplicated managed-root plan
WHEN Mantle prepares and commits the batch
THEN it MUST preserve output order, source labels, registration classes, generation metadata, and source inclusion policy
AND it MUST NOT publish source roots after a failed worker result.

#### Scenario: structural repair would change pipeline meaning

GIVEN a proposed lint repair changes worker execution, cache-only trust, managed registration, retained-root order, failure keys, evidence fields, or a public API
WHEN the repair is reviewed or tested
THEN Mantle MUST reject the repair even if the Tiger Style command exits successfully
AND the finding MUST remain actionable until a semantics-preserving repair passes.

#### Scenario: full check advances beyond the pipeline gate

GIVEN focused pipeline validation and the repository Tiger Style check pass
WHEN local-builder and ordinary `nix flake check -L` run
THEN neither run MUST fail on a `crunch-pipeline` Tiger Style finding
AND any later independent failure MUST remain an exact blocker without disabling or downgrading its gate.
