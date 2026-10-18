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

The original29 selected receipts remain pre-final-format historical evidence.
Separate [post-cleanup milestones](../.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/post-cleanup-milestones-v1.md)
now record actual V2 root/two-generator success, V3 exact generated installation
and root/operator PASS184 plus limited matrix checks, and V4 helper/Android/
store/Casita positive and rejection observations on their distinct subjects.
The existing namespace test now uses the TMPDIR-honoring fixture helper; its
actual body passed without skip. Dedicated public composed-list scenarios proved
nearer selected-prefix authority and rejection, not only lookup/cache behavior.

Actual pinned native strict validate and proposal/design/tasks all returned
exit0/valid:true/issues:[] (gates PASS), with declared17 documentary/policy
inputs unchanged across execution, binding
`943cae0a74f58b0b3a526acbe9feec1b6d665f1927765b289157ae4c980c7dac`.
This is BEFORE these additive outcome records, not a post-edit/full production
tree identity. Full raw native streams and COMPLETE losslessly compact capture
metadata retain all fields and verified exact original-format reconstruction.
Tasks ordering was valid with seven Done/four Todo and P1 blocked by G1/G2/G3
at capture; after recording G3, G1/G2/semantic acceptance still hold P1. No
probe executed. These separately released read-only document/saved-evidence
commands were independent of held product/Nix/builds and did not admit them.

Full ordinary quality, full Nix/TigerStyle and P1 remain PENDING. A later tiny
scratch witness actually passed: fresh `/build` mapped beneath original R/t
through matching dev0:47/zfs/datapool/mantle-store builder/host rows; recovery
task6/controller/inner/monitor child0, no stop. Final shared83564298240B left
187564032B to frozen stop, not proof of full-build fit or successor authority.
Latest G1 admission then refused exit75 BEFORE enqueue/bridge/Clippy/tests:
free40903507968B/shared83789479936B,37617664B over unchanged83751862272B stop.
This is resource refusal, not compiler failure; full G1/G2 remain unexecuted.

Independent existing-cohort static canonical17 v3, extra12 and edition2024
gateway-leaf checks passed on unchanged611 declared source/lock/config inputs
`4df6dbccf81c2c49b7b19b82450e674f720c231b3833e7707af2c74763f4ee91`.
Complete v1/correction/v2/v3 metadata and ten raw streams are separately retained.
v1 canonical dispatch failed before Cargo; historical cause unknown, not proven
login/BASH_ENV interference. v2's helper-init diagnostics invalidate clean PASS
despite exit0; v3 used explicit child environment, no source/profile edits.
Passed extra12/gateway were not rerun. Static formatting is not the full gate.

Fresh unchanged3150 inventory found zero eligible/238 retained, no second apply;
original436/F53A operation is CLOSED. Separately authorized disposal affected
only a brand-new completed consumer created AFTER that operation: all eight
identities passed under preserved Cargo EXLOCK, six files/two empty directories
removed without errors. Source/manifest/ITS copied-lock snapshots/stdout remain
retained; other R resources untouched. Immediate free40859074560B/shared
83833913344B was unchanged and82051072B over stop: physical gain ZERO. Gross
inode267740160B and target-name shrink535475712B (two hardlink names) are not
physical recovery. Neither old refusal nor ZERO gain is retconned by the later
witness; no second cache eviction, funding or causal recovery is inferred.
F0/E0/caps/epoch/pins/profiles remain frozen. Repeat requires fresh admitted
paths/reconstruction, not old exclusive-script replay; no automatic retries.
Final validation of these updated documentary inputs is separately pending.
ADR remains Proposed, not overall semantic/lifecycle acceptance, publication,
release readiness or bootstrap/reproducibility proof.

Rejected alternatives are blanket side selection, duplicate Cargo DTOs, a single
Android ceiling detached from renderer operation, raw-overlay-only listing,
hand-spliced generated identities, and using a new resource epoch to erase
prior usage. The current lifecycle design records their specific failure modes.
