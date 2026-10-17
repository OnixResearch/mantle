# ADR 0114: Reconcile lineage APIs with explicit contract boundaries

- **Status:** Proposed
- **Date:** 2026-10-17

## Context

The current main integration joins local subject
`d20459c6db6d60ed2b08533911e279cbeadb742e` and upstream subject
`d75158b00b080535f2ffea12430665ca3d57ea82`. Both contain required API,
configuration and evidence behavior. A textual merge or duplicate DTO does
not decide which component owns a concept, whether a route's tool supports a
timestamp, or whether store inspection verifies the same signature authority.
The new lifecycle subject is
`.cairn/changes/reconcile-main-api-lineages-2026-10-17/`; earlier campaigns
are not reaccepted. ADRs 0110–0113 retain their existing identity and status.

## Decision drivers

- Preserve meaningful behavior of both parents without parallel legacy APIs.
- Keep functional-core decisions and imperative-shell authority separated.
- Bind timestamp policy to the renderer operation, not a convenient DTO union.
- Make selected-prefix signature authority consistent across get/list/references.
- Keep current generated output and bounded evidence traceable to actual runs.
- Preserve frozen resource accounting and distinguish unknown epochs from proof.

## Decision

Use `UnitBinding.build_script_run: Option<BuildScriptRunBinding>` as the Cargo
run-binding API, retaining links/native input/environment/PATH facts there and
explicit tool runtime roots in `PlanBindings.runtime_inputs`. Reuse extracted
Rust-plan directive handling and migrate producers/helpers/fixtures, rather
than keeping competing script-run fields. Cargo graph authority and ADR 0113
host-unit equivalence rules remain independent of Rust-plan topology.

Keep published Android and `crunch_android_core::reviewed` contracts distinct
where source authority and renderer operations genuinely differ. Both admit
entry epochs from 315532802; published staged-mtime JAR creation admits through
4354819199, whereas reviewed `jar --update --date` ends at 4102444799. Nominal
odd final seconds are represented at ZIP's two-second precision. Do not infer
pinned reviewed JDK/SDK execution from installed OpenJDK option observations.

Use the configured logical prefix for layered get, composed list and reference
signature checks. Listing must preserve overlay/base precedence and fail on
invalid visible trust, not silently report an empty overlay as the whole store.
Casita's destination policy must be reobserved for cache/GC authority; local
action-result publication/discovery does not establish trusted remote reuse.

Generate operator artifacts from current Clap plus reviewed Nickel using the
existing actual generators. Never reconcile catalog identities by hand.

Separate current smoke evidence, final-source checks, historical receipts and
resource epochs. The original pool-availability baseline (F0) and caps remain
frozen. Proven exclusive-growth credit (E) is currently zero; the correction
for uncaptured frees/concurrent allocation (H) remains unknown. Unknown H does
not create a new hard-quota requirement: the original legacy guard with zero
credit already exceeds its threshold. [Current evidence](../.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/current-observations.md)
records the exact baseline and exhausted read-only pre-baseline metadata route.
An owned-process/lock observation is not universal descendant or Nix daemon
emptiness proof. No product resume follows from this document.

## Consequences

Future callers must use the owning Cargo binding and explicit Android route,
not aliases that conceal incompatible authority. A cross-route timestamp or
wrong-prefix signature must fail where its actual contract requires it.
Generated outputs require successful current binaries, not manual provenance.

Selected current helper/authoring/store smokes exist, but precede final scoped
formatting. Direct bound-cohort native validate/proposal/design checks returned
`valid: true` for a documentary snapshot; this does not satisfy final-source G3.
Root/generator build, fresh CLI/generated checks, ordinary quality, extra fmt,
final-source Cairn/tasks and Nix/TigerStyle remain PENDING. This proposed
decision is not implementation acceptance, archived lifecycle state, current
release readiness, a successful merge/push, or bootstrap/reproducibility proof.

Rejected alternatives are blanket side selection, duplicate Cargo DTOs, a single
Android ceiling detached from renderer operation, raw-overlay-only listing,
hand-spliced generated identities, and using a new resource epoch to erase
prior usage. The current lifecycle design records their specific failure modes.
