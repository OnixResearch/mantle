# Composition Roots Specification

## Purpose

Defines the `composition-roots` capability.

## Requirements

### Requirement: Composition plans remain frontend-neutral

r[composition_roots.frontend_neutral_plan] Mantle MUST accept only concrete composition facts and MUST NOT interpret frontend package, provider, inventory, activation, deployment, or rollback semantics.

#### Scenario: External frontend supplies a concrete plan

GIVEN a frontend has selected exact immutable object roots, mount points, limits, and collision decisions
WHEN it submits the composition plan to Mantle
THEN Mantle MUST validate and realize those concrete facts
AND it MUST NOT require the frontend's policy model or module semantics.

#### Scenario: Raw system intent is rejected

GIVEN a request contains unresolved package constraints, provider alternatives, machine roles, tags, activation rules, or deployment generations
WHEN Mantle validates the request
THEN it MUST reject the request as outside the composition-plan contract
AND it MUST NOT silently resolve that intent in Mantle core.

### Requirement: Semantic plan identity is canonical

r[composition_roots.canonical_plan_identity] Mantle MUST compute a domain-separated BLAKE3 `plan_ref` from normalized merge semantics and derived binding references. Input encoding, list order, physical paths, resource limits, source envelopes, caller labels, and display metadata MUST NOT affect that identity. Mantle MUST compute a separate `realization_policy_ref` for named limits and admission settings.

#### Scenario: Equivalent projections have one identity

GIVEN two accepted projections contain the same schema, root references, mount points, and collision decisions in different list orders or with different caller labels
WHEN Mantle normalizes both projections
THEN it MUST compute the same canonical bytes and `plan_ref`
AND later optional adapters MUST map equivalent values to that same core identity.

#### Scenario: Semantic change alters identity

GIVEN one accepted plan changes an input root reference, mount point, collision decision, or merge-policy version
WHEN Mantle normalizes the changed plan
THEN it MUST compute a different `plan_ref`
AND it MUST NOT reuse a realization receipt from the prior plan.

#### Scenario: Resource policy has a separate identity

GIVEN two requests have the same merge semantics but different named resource limits or admission settings
WHEN Mantle normalizes both requests
THEN it MUST compute the same `plan_ref` and different `realization_policy_ref` values
AND each realization receipt MUST bind the policy identity that governed its attempt.

#### Scenario: Store prefixes do not enter the plan

GIVEN equivalent object roots were discovered through different Nix-compatible or Mantle store prefixes
WHEN a caller constructs the concrete path-free plan
THEN the prefixes MUST NOT appear in canonical plan fields
AND they MUST NOT change `plan_ref` or the resulting castore root.

### Requirement: Composition planning has a pure bounded core

r[composition_roots.pure_bounded_core] Mantle MUST compute path validation, merge decisions, conflict outcomes, canonical identity, and receipt preimages in a pure deterministic core over supplied in-memory facts.

#### Scenario: Complete bounded snapshot produces a plan

GIVEN the shell supplies every referenced directory and blob fact within named entry, depth, path, file-byte, total-byte, binding, and conflict limits
WHEN the core plans composition
THEN it MUST return deterministic directory facts, limit usage, conflict outcomes, and a receipt preimage
AND it MUST perform no filesystem, store, environment, process, network, clock, random, or presentation effects.

#### Scenario: Limit violation fails before persistence

GIVEN a plan or supplied snapshot exceeds any named bound
WHEN the core validates it
THEN it MUST return a stable limit diagnostic
AND the shell MUST NOT persist a partial composition root or successful receipt.

### Requirement: Logical paths fail closed

r[composition_roots.logical_path_safety] Mantle MUST normalize root-relative mount points and MUST reject absolute components, parent traversal, empty internal components, platform prefixes, and ambiguous path forms.

#### Scenario: Safe mount target is accepted

GIVEN a binding uses the explicit root target or normalized relative components such as `usr`, `bin`
WHEN Mantle validates the plan
THEN it MUST admit the logical target subject to all other checks
AND canonical identity MUST use the normalized target.

#### Scenario: Unsafe mount target is rejected

GIVEN a binding uses an absolute path, parent component, empty internal component, drive prefix, or non-normalized equivalent
WHEN Mantle validates the plan
THEN it MUST reject the plan before object loading
AND it MUST identify the failing binding and path class without exposing unrelated host paths.

#### Scenario: Symlink targets remain opaque

GIVEN an input tree contains a symlink node
WHEN Mantle plans and realizes the composition
THEN it MUST preserve the supported symlink target bytes without following the target
AND the receipt MUST NOT claim host-path confinement or safe later materialization from that fact alone.

### Requirement: Conflicts require explicit decisions

r[composition_roots.explicit_conflicts] Mantle MUST derive each `binding_ref` from its exact root reference and normalized mount target. Mantle MUST make merge conflicts independent of input order and caller labels. Identical leaves MAY deduplicate, while each non-identical leaf collision MUST have one exact applicable decision.

#### Scenario: Directories merge and identical leaves deduplicate

GIVEN several bindings contribute directories under the same logical path and some leaves have identical node identity
WHEN Mantle plans the merge
THEN it MUST merge directory children recursively and deduplicate identical leaves
AND it MUST record deterministic merge and deduplication outcomes.

#### Scenario: Non-identical collision selects one derived binding reference

GIVEN two or more bindings contribute different leaves at one logical path
AND the plan contains one exact decision selecting a contributing derived `binding_ref`
WHEN Mantle plans the merge
THEN it MUST select only that binding's leaf for the path
AND it MUST record every displaced binding reference in the conflict outcome.

#### Scenario: Missing or stale decision fails

GIVEN a non-identical collision has no decision, has multiple decisions, names a non-contributing binding, or names a path without a current collision
WHEN Mantle validates the merge
THEN it MUST reject the plan with a stable unresolved or stale decision diagnostic
AND binding order MUST NOT select a winner.

### Requirement: Realization uses complete castore objects

r[composition_roots.castore_realization] Mantle MUST realize a composition only from complete admitted castore object graphs and MUST use the resulting castore directory root object reference as filesystem identity.

#### Scenario: Complete inputs produce one immutable root

GIVEN every input root and referenced object is present and consistent
WHEN the shell applies the pure merge plan and persists its directory facts
THEN it MUST recheck the resulting object graph and return one immutable castore root reference
AND it MUST NOT create a second digest that claims to replace the castore root identity.

#### Scenario: Incomplete input fails without successful output

GIVEN an input root, child directory, or blob is absent, corrupt, or inconsistent
WHEN the shell loads or rechecks the object graph
THEN it MUST fail with a stable completeness diagnostic
AND it MUST NOT publish a successful root or realization receipt.

### Requirement: Realization receipts bind plan and root

r[composition_roots.realization_receipt] Mantle MUST emit a deterministic receipt that binds the plan reference, realization-policy reference, input root references, merge-policy version, conflict outcomes, limit usage, and resulting castore root reference.

#### Scenario: Successful receipt is inspectable

GIVEN a composition realizes successfully
WHEN Mantle emits its receipt
THEN a caller MUST be able to identify the exact plan and resulting root
AND the receipt MUST distinguish deduplication, explicit replacement, and conflict-free merge outcomes.

#### Scenario: Receipt does not broaden claims

GIVEN a realization receipt is valid
WHEN another layer consumes it
THEN the receipt MUST claim only bounded composition identity, linkage, validation, and realization facts
AND it MUST NOT claim ABI compatibility, dependency completeness, runtime correctness, bootability, authorization, deployment success, or release eligibility.

### Requirement: Adapters remain optional

r[composition_roots.optional_adapters] Mantle core MUST NOT require Kamacite, Preserves, OnixOS, OCI, or Nix-compatible paths to compute or realize a composition plan.

#### Scenario: Generic projection reaches the core

GIVEN a supported shell adapter decodes a bounded projection into the generic core model
WHEN Mantle computes identity and realization
THEN the adapter's source format MUST NOT replace the core semantic contract
AND format-specific artifact identities MUST remain separate evidence.

#### Scenario: Optional Preserves adapter is deferred safely

GIVEN Kamacite later defines a Preserves mapping for the stable composition-plan schema
WHEN that adapter is proposed
THEN it MUST use a separate Cairn change with positive and negative mapping fixtures
AND Mantle MUST remain usable without that adapter.

### Requirement: Composition remains experimental and non-deploying

r[composition_roots.experimental_boundary] Mantle MUST keep the initial composition-root surface experimental and MUST stop at root realization, inspection, and export.

#### Scenario: Realization does not activate a system

GIVEN Mantle produced a valid composition root
WHEN the initial command completes
THEN it MUST NOT deploy, activate, switch, roll back, or authorize that root
AND those operations MUST remain owned by an external consumer.

#### Scenario: Unsupported Unix metadata blocks stronger support

GIVEN a consumer requires ownership, ACLs, capabilities, security xattrs, hard links, or device nodes that the selected castore model cannot preserve
WHEN support is evaluated
THEN Mantle MUST report the unsupported metadata classes
AND it MUST NOT promote the experimental root to production system-root support.
