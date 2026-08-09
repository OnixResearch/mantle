# Composition-root Cairn scaffold review

## Goal

Record the smallest honest lifecycle change for frontend-neutral filesystem composition.

Completion requires one active Mantle package with proposal, design, tasks, delta requirements, and an ADR. The package must pass current Cairn structure and stage gates.

## False completion cases

The review rejects these results:

- a Mantle plan that contains OnixOS package, provider, inventory, activation, deployment, or rollback semantics;
- a required Preserves, Kamacite, OnixOS, OCI, or Nix-path dependency in Mantle core;
- a plan identity based on raw transport bytes, list order, physical store prefixes, resource limits, or caller labels;
- hidden last-writer-wins conflict policy;
- a second tree digest that competes with the castore root;
- claims of ABI safety, complete dependency discovery, full Unix metadata, deployment, or release readiness;
- extra cross-repository Cairns that invent mappings to an unstable Mantle schema.

## Audit risks

The main risks are owner confusion, duplicate active work, identity-domain collapse, incomplete object graphs, hidden collision policy, path traversal, unbounded snapshots, and premature runtime claims.

## Search budget

The review used four architecture families across Mantle, Kamacite, OnixOS, and `bounded-tree`. It used repository READMEs, local agent notes, accepted specs, active changes, ADRs, and direct source findings from the prior architecture review.

The search stopped after direct repository inspection found one bounded owner and no existing composition-root change.

## Approach registry

| Family | Mechanism | Result | Evidence | State |
|---|---|---|---|---|
| Mantle-only core contract | Define concrete object-root composition and defer adapters | Smallest complete owner boundary | ADR 0010, ADR 0024, `cairn/specs/build-tool-boundary/spec.md`, no existing composition-root package | validated |
| Immediate Kamacite package | Define Preserves mapping before the Mantle schema is stable | Would create avoidable schema churn and imply a required integration | Kamacite owns optional interchange; Mantle ADR 0019 already shows the opaque evidence pattern | blocked |
| Immediate OnixOS package | Define package selection and deployment lowering before generic realization exists | Would couple policy to an unimplemented lower boundary | ADR 0010 assigns lowering to the frontend after Mantle has a concrete contract | blocked |
| Move the merge to `bounded-tree` | Reuse product-neutral filesystem observation and copy mechanics | Does not own castore object graphs or Mantle root identity | `bounded-tree/README.md` keeps product identities consumer-owned and focuses on observed filesystem trees | falsified for this change |

A secondary model review recommended avoiding duplicate packages. Direct repository search did not find an existing package that owns this composition-root contract.

## Surviving design

Mantle receives exact castore roots, normalized mount points, named limits, and explicit collision decisions. A pure core derives binding references, computes separate plan and realization-policy identities, and returns merge facts. A thin store shell produces one castore root and a bounded receipt.

Kamacite and OnixOS remain possible later consumers. Neither is a dependency or schema owner for this change.

## Adversarial checks

- Binding order and caller labels cannot select a conflict winner.
- Store prefixes, transport bytes, caller labels, and resource limits cannot enter `plan_ref`.
- Named limits and admission settings remain visible through `realization_policy_ref`.
- Symlink targets remain opaque and are never followed during planning.
- Missing child objects block realization.
- Full Unix metadata gaps block production system-root claims.
- Action-result and execution-profile changes remain separate lifecycle decisions.

## Baseline lifecycle result

Before scaffold creation, the default repository policy failed to parse with the current Cairn CLI:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

This is an existing repository-policy freshness blocker. Validation for this scaffold uses Cairn's current generated policy explicitly, matching other active Mantle changes.

## Non-claims

This review does not prove that the merger is implemented, that castore supports complete Unix metadata, or that a composed root executes safely.

It does not authorize Kamacite, OnixOS, deployment, activation, rollback, or production support.

## Scaffold validation

Pueue task `7661` passed `git diff --check`, full Cairn validation, and the proposal, design, and tasks gates.

The commands used Cairn's current generated policy explicitly because the repository-local generated policy has the baseline freshness blocker above.

All four Cairn results reported `valid: true` and `verdict: "PASS"`.
