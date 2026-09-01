# Root action-trust architecture search

## Success contract

### Goal

Emit one complete root action-trust plan before any current-attempt build action. Reconcile every protected execution and build record against that plan.

### Completion evidence

- Every action has one stable ID, broad stage, producer edges, input authority, outputs, executable authority, locality, count bounds, and resources.
- Each provider action was governed by a pre-execution stage plan retained in its promoted checkpoint.
- A restored final attempt composes those origin plans before stage1.
- Stage1 and stage2 Rust unit plans exist before their executions.
- Stage1 recreates the stage2 plan and matches the precommitted identity.
- The final receipt binds root-plan and reconciliation BLAKE3 values.
- `bootstrap trust-report` reports `complete` with no blockers.

### False completions

- Rendering a view over the six broad stages.
- Counting post-execution records without a prior plan.
- Treating a generated path as executable authority.
- Treating zero reported events as zero unmatched events.
- Adding action files to V48 after publication.
- Importing the existing checkpoint as if it contained action plans.

### Risks

The main risks are circular planning through produced rustc or stage1 Mantle, stale absolute paths, duplicate aliases, post-hoc authority, hidden dynamic derivations, and cache-only completion.

### Budget

This search used one serial architecture round, existing checked evidence, and targeted source reads. It did not launch a new provider proof.

Allowed outcomes were validated, blocked, exhausted, or user-decision-required.

## Approach registry

### Family: operator-view-only

- **Mechanism:** Derive trust from the existing v2 receipt and six stage records.
- **Claim:** The current receipt proves complete child-action authority.
- **Artifact:** V49 trust report.
- **Evidence:** V49 reports `fixed-point-only` and three blockers.
- **Gap strength:** weaker.
- **Blocker:** No child-action plan or reconciliation digest exists.
- **Next check:** None. This route is falsified.
- **State:** falsified.

### Family: broad-stage-actions

- **Mechanism:** Treat each of the six proof stages as one action.
- **Claim:** Six matched stage records equal full action coverage.
- **Artifact:** `source-built-stage-evidence.json`.
- **Evidence:** StageX alone contains 76,542 protected execution events. Each Rust stage contains 789 unit executions.
- **Gap strength:** weaker.
- **Blocker:** Child executables, producers, inputs, and event bounds disappear.
- **Next check:** None. This route is falsified.
- **State:** falsified.

### Family: retrospective-aggregation

- **Mechanism:** Build a root plan from V48 observations after execution.
- **Claim:** Post-execution enumeration can become pre-execution authority.
- **Artifact:** V48 StageX audit, native build report, Rust receipts, and Rust topology receipts.
- **Evidence:** The files can enumerate many actions, but the V48 receipt does not bind a root plan.
- **Gap strength:** weaker.
- **Blocker:** The mechanism invents authority after the actions ran.
- **Next check:** None. This route is falsified.
- **State:** falsified.

### Family: one cold eager plan

- **Mechanism:** Enumerate every provider and Mantle action before the first provider action.
- **Claim:** One plan precedes the complete cold proof.
- **Artifact:** StageX materialization plan, native derivation conversion cache, Rust provider bootstrap plans, and Rust unit derivation graph.
- **Evidence:** StageX and native actions can be planned eagerly. Stage1 Rust planning needs the Rust provider that the proof has not built yet.
- **Gap strength:** stronger.
- **Blocker:** The requested plan depends on a produced rustc. Closing that dependency before all execution is equivalent to already having the provider.
- **Next check:** None without transferring provider authority into inputs.
- **State:** blocked.

### Family: promoted staged plans

- **Mechanism:** Emit complete stage-local plans before each provider stage, bind them into a new promoted checkpoint, then compose one root plan before current stage1 execution.
- **Claim:** Every restored action retains its original pre-execution plan, while every current action appears in the root plan before it executes.
- **Artifact:** New provider checkpoint payloads, root action plan, stage1 Rust plan, stage1-produced stage2 plan, and reconciliation.
- **Evidence:** StageX already emits `transition-plan.json`. Native eager conversion now exposes every transitive derivation. Preserved replay matches 88 unique actions to 568 bounded worker observations across seven roots. Rust provider construction emits bounded stage plans. Rust-plan capture exposes every unit derivation before topology execution.
- **Gap strength:** equivalent.
- **Blocker:** Rust provider scripts and Rust build scripts can spawn child compilers and linkers without a root-bound execution audit. The current checkpoint also lacks the new native action payloads.
- **Next check:** Put Rust provider and Rust-unit child execution under producer-bound interception, then reconcile one positive and one denied child fixture.
- **State:** active.

## Adversarial audit

The selected route must reject the current checkpoint. Accepting it would be post-hoc authority. Each action-plan payload must be immutable, remeasured, and stage-specific. Root composition must reject missing adapters, duplicate action IDs, unknown producer IDs, path-only executable authority, remote locality, cache-only completion, and count overflow.

Stage2 remains stage1-owned. The host may precompute a candidate unit plan after checkpoint restore, but stage1 must recreate and match that plan before stage2 execution.

## Decision

Select promoted staged plans. Do not weaken the requirement to broad stages or retrospective aggregation.

Implementation commit `33f92fd3` adds the native eager-plan adapter. It rejects hidden derivation files, runtime dynamic actions, unknown producers, unbound builders, event overrun, and action drift.

The next implementation boundary is Rust child execution interception. Do not publish a new checkpoint until that audit is bound.
