# Design: Bind static inputs to dynamic-plan roots

## Goal and scope

A statically evaluated derivation consumes the output of a dynamic-plan root.
Its identity is known at evaluation. The concrete unit is bound after the
producer's plan is accepted and substituted when the consumer is dispatched.

## Current behavior

- Nickel inputs are `Source`, `DerivationFile`, `ResolvedDerivation`,
  `OutputSelection`, and `Derivation` (`crates/crunch-glue/src/types.rs:191-202`).
- Plan roots are wanted as requested roots after plan acceptance
  (`crates/crunch-build/src/worker.rs:1638-1654`, `1046-1051`), and root
  outputs are exported.
- `Goal::new_awaiting` and `GoalState::AwaitingDerivation` exist for goals
  whose derivation comes from a producer, but no production path creates them
  (`crates/crunch-build/src/goal.rs:26-39`, `100-114`).
- Build requests already replace Mantle and Nix-compatible output placeholders
  in arguments and environment (`crates/crunch-build/src/build_request.rs:1219-1259`).
- Sandbox inputs are the direct input paths plus their reference closures
  (`crates/crunch-build/src/orchestrate.rs:1050-1076`).

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Import-from-derivation | Evaluate again after the producer runs | Rejected by ADR 0011: evaluator suspension and unbounded import | None |
| Two runs | Build the producer, then evaluate consumers against reported paths | Rejected: not composable and not cacheable by identity | None |
| Plan-internal consumers | Put every consumer inside the plan | Rejected as the general answer: static Nickel packaging cannot be expressed | None |
| Deferred consumer identity | Consumer output paths become known only after resolution | Rejected for now: every downstream derivation would defer as well | Revisit with content-addressed resolution |
| Request identity with late binding | Consumer identity binds the symbolic reference; the worker binds at plan acceptance and substitutes at dispatch | Selected | Identity and binding fixtures |

## Contract and component ownership

- **Nickel contract** (`lib/contracts.ncl`, `lib/derivation.ncl`): a
  `plan_output` input record with `name`, `producer`, `plan_output`,
  `root`, and `unit_output` fields. `name` identifies the
  `{{mantle-plan-output:<name>}}` marker in consumer arguments and
  environment values. Names use lowercase ASCII identifiers, at most
  64 bytes; root ids and output names retain the v1/v2 plan grammar.
- **Pure conversion** (`crunch-glue`): validates the producer declares
  `plan_output` in `dynamic_plan_outputs`, records an input edge on
  that output, and writes one compact canonical JSON array into
  `__MANTLE_PLAN_OUTPUT_BINDINGS` in the consumer environment. Each
  record has exactly `name`, `producer_drv_path`, `plan_output`,
  `root`, `unit_output`, and `placeholder`; records sort by `name`.
  The placeholder is `nix_compat::store_path::hash_placeholder` of
  `mantle-plan-output:<name>`. Conversion replaces all declared markers
  with those deterministic placeholders and rejects unbound markers.
  The named limit is 16 distinct references per consumer; a derivation
  with none retains its exact prior environment and identity.
- **Pure binding core** (`crunch-build`): decodes the canonical record,
  validates its declared producer edge, binds the named unit only if it
  is in the accepted plan's roots, and returns the root derivation path
  and selected output or a typed failure without I/O.
- **Shell** (worker and build request): waits for the producer, applies
  the binding by adding a wait on the root unit, replaces placeholders
  at dispatch, adds the bound path to sandbox inputs and reference-scan
  needles, and records provenance.

## Decisions

### Decision: Request identity instead of deferred identity

**Choice:** The consumer's derivation hash covers the producer derivation path,
plan output, root unit id, unit output, and local input name through its
ordinary input edge and canonical binding record. Its output paths are
computed at evaluation like any input-addressed derivation.

**Rationale:** The producer derivation path fixes the plan request the same
way an input derivation path fixes an ordinary dependency. Deferring the
consumer's identity would force every downstream derivation to defer too. The
determinism assumption matches the one input-addressed builds already make,
and reports bind the concrete plan digest for review.

### Decision: No derivation format change

**Choice:** Encode the reference as an input edge on the producer's declared
plan output plus a canonical binding record in the environment, not as a new
ATerm field.

**Rationale:** Nix compatibility boundaries and existing parsers stay
unchanged, and the binding record enters the derivation hash through existing
fields.

### Decision: Bind at plan acceptance, substitute at dispatch

**Choice:** When the producer's plan is accepted, the worker resolves the root
unit and adds a wait edge from the consumer to it. Placeholder replacement
happens when the consumer is dispatched, after the root has completed.

**Rationale:** Waiting for the root means content-addressed roots already have
realized paths at dispatch, so this change does not depend on resolved
content-addressed identity.

### Decision: Only plan roots are bindable

**Choice:** A reference names a unit listed in the plan's `roots`.

**Rationale:** Roots are the plan's export surface. Binding internal units
would couple consumers to producer internals.

## Failure behavior and ordering

Stable failure reasons are `plan-output-producer-failed`,
`plan-output-plan-rejected`, `plan-output-undeclared`,
`plan-output-root-missing`, `plan-output-output-missing`, and
`plan-output-root-failed`. Evaluation also rejects duplicate/invalid
names, an undeclared marker, and more than 16 references. A malformed
canonical binding record or a still-unbound dispatch placeholder is a
fail-closed error. Failures follow stage precedence: a producer failure
precedes plan rejection; on accepted plans root membership precedes output
membership, which precedes a later failed root. For several references,
the first failure in canonical input-name order wins. A failed binding
marks the consumer failed and propagates to its dependents through the
existing failure path; it never emits a successful consumer output.

## Tests

- Positive: a consumer reads a root's file through the placeholder; an
  input-addressed and a content-addressed root both bind; consumer derivation
  paths are stable across evaluations; an unchanged second build reuses the
  consumer without executing it.
- Negative: a rejected plan, an undeclared plan output, an unknown root, an
  unknown output, a failed root, over-limit references, and a marker without a
  matching input.
- Compatibility: derivations without the input keep their identities.

## Risks / Trade-offs

- A nondeterministic producer can bind different content under one consumer
  identity, as with any input-addressed dependency. Reports bind the plan
  digest, so drift is visible.
- Late wait edges add a goal transition. Bounded references and the existing
  waiter bookkeeping limit the change.

## Claim boundary

Binding evidence proves the recorded reference, plan acceptance, bound paths,
and placeholder replacement. It does not prove producer determinism, root
correctness, consumer correctness, or release eligibility.
