# Build Correctness Specification

## Purpose

Define explicit identity and claim boundaries for immutable cache snapshots and mutable tool workspaces.

## Requirements

### Requirement: Stateful workspace execution modes are explicit [r[build_correctness.stateful_workspace_modes]]

Mantle MUST require an explicit workspace mode when retained tool state is available. `none` MUST use no retained workspace state; `immutable-snapshot` MUST use declared read-only content-addressed snapshot objects that participate in action identity; and `mutable-session` MUST use a bounded leased writable workspace whose compatibility and authority are validated before sandbox start. Mantle MUST NOT silently fall back from one mode to another.

#### Scenario: Immutable snapshot is a declared input

- GIVEN an action declares a compatible immutable workspace snapshot by canonical object ref and stable guest mount path
- WHEN Mantle constructs and executes the action
- THEN the snapshot ref and mount declaration MUST participate in action identity and ordinary input admission
- AND host storage paths or prior mutable workspace identity MUST NOT affect that identity.

#### Scenario: Mutable workspace requires compatible lease

- GIVEN an action requests mutable-session mode
- WHEN Mantle validates the workspace
- THEN worker, authority class, action compatibility, toolchain refs, guest path, current job/attempt/fence, and quota policy MUST match
- AND any mismatch MUST reject workspace reuse before sandbox execution.

#### Scenario: Mode fallback is not implicit

- GIVEN the requested snapshot is missing or the mutable workspace is unavailable, quarantined, stale, or over quota
- WHEN Mantle plans execution
- THEN it MUST fail or choose another mode only under an explicit configured fallback
- AND the report MUST identify the actual mode and fallback reason.

### Requirement: Mutable workspace execution has a narrower claim boundary [r[build_correctness.mutable_workspace_claim_boundary]]

Mantle MUST classify mutable-session workspace content as execution history rather than a declared immutable action input. An execution that reads mutable workspace state MUST NOT by itself publish or satisfy a strong shared action result. A separate clean execution from equivalent declared inputs MAY provide comparison evidence when its admitted output object set matches, but it MUST NOT retroactively relabel the original mutable execution as hermetic.

#### Scenario: Warm build reports narrower evidence

- GIVEN a build reads a compatible mutable leased workspace and produces outputs that pass ordinary content and output admission
- WHEN Mantle reports the result
- THEN it MAY report successful practical execution and admitted output objects
- AND it MUST state that mutable workspace history prevents strong hermetic shared-reuse admission from that run alone.

#### Scenario: Mutable result is excluded from shared action cache

- GIVEN an action result was produced using mutable-session mode without accepted clean-rebuild comparison evidence
- WHEN Mantle considers shared action-result publication or strong reuse
- THEN it MUST reject that candidate with a stable mutable-state reason
- AND it MUST NOT treat matching output content alone as proof that workspace history was irrelevant.

#### Scenario: Clean comparison records equivalence narrowly

- GIVEN a separate `none` or declared immutable-snapshot rebuild uses equivalent declared action inputs and produces the same admitted output object set as a warm build
- WHEN Mantle evaluates comparison evidence
- THEN it MAY record output-set agreement bound to both executions
- AND it MUST NOT claim general tool-cache correctness, future determinism, or hermeticity of the original warm execution.

#### Scenario: Sensitive or escaping state is quarantined

- GIVEN a workspace contains policy-defined secret material, host-path leakage, path traversal, escaping symlinks, incompatible ownership, or content that cannot be scrubbed within bounds
- WHEN Mantle prepares reuse or snapshotting
- THEN it MUST reject and quarantine the workspace before another action reads it
- AND no shared snapshot or action result may reference the rejected state.
