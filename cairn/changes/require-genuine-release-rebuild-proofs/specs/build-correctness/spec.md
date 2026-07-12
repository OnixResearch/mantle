# Build Correctness Specification

## Purpose

Require deterministic release proofs to rebuild selected artifacts from content-bound declared inputs without read authority over the published outputs.

## Requirements

### Requirement: Release determinism uses a genuine rebuild

r[mantle.build_correctness.release_determinism.genuine_rebuild] Mantle MUST emit a promoting deterministic release proof only when every proof run executes a reviewed rebuild recipe over declared source and toolchain inputs without read authority over the published target artifacts, content-identical aliases, prior proof outputs, or the ordinary reproduce output.

#### Scenario: Declared source rebuilds converge

r[mantle.build_correctness.release_determinism.fixtures.positive]
- GIVEN two proof runs receive the same declared source, recipe, toolchain, provider, sandbox, and normalization identities through distinct fresh stores
- AND neither run can read any published target identity
- WHEN both runs produce the exact selected artifact set with matching BLAKE3 digests
- THEN Mantle MAY classify the bounded result as a genuine `self-rebuild-match` contribution.

#### Scenario: Copying the target cannot promote

r[mantle.build_correctness.release_determinism.fixtures.negative.target_copy]
- GIVEN a rebuild helper attempts to copy the published binary directly or through a content-identical alternate path, symlink, hardlink, proof-bundle entry, or prior output
- WHEN Mantle plans or executes a deterministic proof run
- THEN the target bytes MUST be unavailable to the rebuild process
- AND the proof MUST fail with a deterministic target-authority or undeclared-input blocker.

### Requirement: Rebuild identity is content-bound

r[mantle.build_correctness.release_determinism.identity_binding] Mantle MUST bind canonical recipe bytes, executable and tool byte identities, ordered arguments, source and input closure identities, provider and target identities, sandbox policy, effect policy, normalization policy, and approved read/write roots into a versioned BLAKE3 rebuild descriptor cited by every proof run.

#### Scenario: Recipe or tool drift fails closed

r[mantle.build_correctness.release_determinism.fixtures.negative.identity_drift]
- GIVEN a recipe, executable, tool, source closure, argument, provider, or policy changes while a path or label remains unchanged
- WHEN a proof run is compared with its accepted rebuild descriptor
- THEN the content-bound identity MUST differ or validation MUST fail
- AND the stale descriptor MUST NOT contribute to a promoting verdict.

### Requirement: Rebuild authority planning is pure

r[mantle.build_correctness.release_determinism.authority_plan] Mantle MUST decide allowed rebuild inputs, target-identity exclusions, run-root separation, and deterministic proof eligibility in a pure core over normalized observations, while byte measurement, capability-root setup, sandbox execution, output comparison, and receipt writing remain in the shell.

#### Scenario: Authority plan is testable without execution

r[mantle.build_correctness.release_determinism.authority_plan.test]
- GIVEN in-memory published target identities, candidate input observations, run roots, and policy
- WHEN the authority planner evaluates them
- THEN it MUST deterministically return an allowed plan or ordered blockers without reading files, environment state, clocks, networks, or processes.
