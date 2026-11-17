## Phase 1: Proof integration

- [x] [depends:bind-full-source-rust-provider] I1 Define a pure full-bootstrap proof plan over authenticated source authority, StageX lineage, native/Rust provider construction, immutable closure policy, Cargo-free stage execution, strict hermeticity, protected execution, bounded resources, and v2 receipt fields. r[bootstrap_inventory.source_built_mantle_fixed_point]
  - Evidence: `src/source_built_fixed_point.rs` defines the domain-separated six-stage plan, exact source roles, empty-output precondition, strict policy, resource bounds, stage1-owned stage2 orchestration, v2 receipt-field contract, and bounded non-claims. Focused validation passed 9 positive and negative tests plus strict first-party Clippy in pueue task `4428`. ADR 0050 records the composition boundary.
- [x] [depends:materialize-stagex-lineage-provider] I2 Extend source-bundle export/hydration so a fresh proof root receives every declared native and Rust source record plus vendor inputs, starts with empty provider/Mantle output authorities, and cannot admit provider outputs as proof inputs. r[bootstrap_inventory.source_built_mantle_fixed_point]
  - Evidence: `source-built-fixed-point` source-bundle mode labels the audited hex0 seed and validated StageX lineage as source authority, requires materialized native fetch records, Mantle source, vendor inputs, and Rust proof inputs, and rejects a provider manifest substitution. Hydration imports and pins every record, publishes only `vendor-deps/`, and leaves transition/provider/output directories absent. Four focused positive and negative tests passed; the full 70-test source-bundle suite and strict first-party Clippy passed in pueue tasks `4476`, `4470`, and `4473`.
- [ ] [serial] I3 Implement the shell that constructs the StageX native provider and full-source-bound Rust provider from those sources, then builds stage1 Mantle through the receipt-bound Cargo-free topology under strict no-fetch/no-fallback policy. Before execution, emit a complete root-scoped action trust plan with fixed or producer-linked executable authority, input authority, producer edges, local-only execution, event-count bounds, and resource limits. Reject incomplete action adapters and path-only generated authority. r[bootstrap_inventory.source_built_mantle_fixed_point]
- [ ] [serial] I4 Invoke the produced stage1 Mantle to build stage2 through the same plan and closure policy, compare BLAKE3 binary digests, and reconcile every protected-exec and build execution record with the planned action, executable, producer, locality, and count bounds. Fail closed on unknown or missing events, authority, closure, effect, smoke, or digest drift. r[bootstrap_inventory.source_built_mantle_fixed_point]
- [ ] [serial] I5 Emit and verify `mantle-deterministic-proof-receipt-v2`, durable failed-attempt evidence, provider-kind linkage, action-trust plan and observation digests, planned-versus-observed reconciliation, release-evidence inputs, and atomic successful-proof alias updates. Expose the bound result through `mantle --json bootstrap trust-report --proof-root <path>`. r[bootstrap_inventory.source_built_mantle_fixed_point]

## Phase 2: Verification

- [x] [serial] V1 Add positive pure-plan, action-trust, reconciliation, and receipt tests. Add negative tests for missing source records, nonempty initial provider/output authority, imported or cache-hit provider completion, incomplete action adapters, path-only generated authority, producerless generated executables, unknown or missing execution events, executable digest drift, producer drift, event-count overflow, remote or cache-only execution, undeclared input authority, v1 receipt, wrong provider kind, closure drift, Cargo invocation, live fetch, ambient tool/read, protected-exec violation, practical fallback, stage2 host orchestration, output mismatch, timeout, disk preflight failure, and partial alias publication. Run focused `cargo_free_self_build`, `source_toolchain_closure`, `self_build`, release determinism, and bootstrap parity tests. r[bootstrap_inventory.source_built_mantle_fixed_point]
  - Evidence: complete V1 boundary coverage and source-bound qualification are retained in `evidence/2026-11-12-source-input-binding-corrections.md`; latest Leviathan tasks 1557 and 1559 passed 259 source-built tests plus the receipt, CLI, Cargo-free, closure, self-build and release-core legs on frozen source `22eb45ca5321416fce3278a228a43224dd0026c5`. These mechanism tests do not accept I3–I5, real bootstrap parity, release evidence or the cold fixed point.
- [ ] [serial] V2 Run the full proof on Leviathan (`leviathan.cymric-daggertooth.ts.net`) as the sole execution host. Use a fresh hydrated source root and set `CRUNCH_NO_FUSE=1`. Before launch, retain exact source and profile transfer parity. Also retain observed host, system, architecture, kernel, resource, sandbox-tool, and pueue-task facts. Use SSH, rsync, and pueue only for outer transfer, launch, observation, and evidence retrieval. Do not use Mantle `--builder`, Nix remote-action dispatch, split-host stages, or route-name trust. Preserve exact provider, closure, stage, smoke, action-trust, reconciliation, audit, v2 receipt, and BLAKE3 evidence. Require stage1/stage2 binary equality and complete planned-versus-observed action coverage. Require all proof actions to execute locally on Leviathan. Require zero unknown events, missing events, live fetches, Cargo invocations, or fallback events. Require strict hermeticity. r[bootstrap_inventory.source_built_mantle_fixed_point]

- [ ] [serial] V3 Run `nix develop -c cargo test -p mantle --test release_cli`, `nix develop -c cargo test -p mantle --test bootstrap_parity_cli`, focused action-trust core and CLI tests, the machine-contract checker, `nix develop -c cargo fmt --check -p mantle -v`, focused first-party Clippy, `git diff --check`, release-evidence verification, Cairn validation, and all three gates; do not mark the proof complete from preflight, a partial trust report, provider smoke, one-shot build, or a mismatched fixed point. r[bootstrap_inventory.source_built_mantle_fixed_point]

## Current source handoff (2026-10-23)

No build, test, formatter, or proof was run by this source/provider slice.
Earlier receipts and checked historical tasks do not accept the current tree.
`src/source_built_fixed_point_shell.rs` now validates all declared resource
bounds before hydration, and the current cold route rejects inherited compiler
cache/scheduling authority. See the current source-built section of
`docs/operator-proof-guide.md` for the two distinct preparation/proof commands,
profile identity handoff, 500 GiB example budget, and independent receipt and
parity gates. Preparation must start from unexpanded authenticated source and
publishes `source-profile.json` plus `source-program-preparation.json`; its
output profile identity replaces the input profile identity at proof launch.

Current host access is not the old blocker: Leviathan is reachable through the
configured SSH route. Its retained `proof-inputs/` contains StageX, native, and
Rust host-tool source manifests and Rust archive inputs. Those retained inputs
are not a qualified current cohort: the final frozen Mantle/lock/vendor source
must be rebound, every current source declaration covered, and inert programs
prepared and reviewed before the cold proof. The historical native output pin
in `bootstrap/evidence/full-source-provider-admission.json` remains an explicit
expected value, not an observation of this tree. Do not replace it with an
unreviewed output observed from the same run merely to obtain success.

The integration owner must run the source-core/source-bundle/source-observation
and source-built shell checks, then execute the real Leviathan cold proof and
validate its v2 receipt, complete action reconciliation, stage1/stage2 binary
equality, provider/closure links, and release evidence. I3–I5 and V1–V3 remain
unaccepted until that current evidence exists; old wrappers, provider-only
smokes, and successful source preparation do not substitute for it.

## Source-authoring handoff (2026-11-06)

The resolver and successor cc overlay repairs have focused test and real helper
execution evidence in [bounded minicargo resolution](evidence/2026-11-06-bounded-minicargo-resolution.md).
Real preparation now stops at the macro-continuation capacity boundary, not the
earlier version search or deleted-transformer panic. I3–I5 and V1–V3 remain
unchecked: this handoff does not accept preparation, a provider, or a cold proof.

## Compact macro handoff (2026-11-06)

The compact controller and explicit deferred-command namespace handoff have
author/protocol tests, real protected execution and focused Clippy/build evidence
in [compact macro continuations](evidence/2026-11-06-compact-macro-continuations.md).
Pueue 64 reran real preparation with unchanged profile pins and resource bounds;
it passed the prior macro-capacity boundary and now rejects ambiguous original
vendor identity `memchr 2.7.1` in successor source topology. No prepared profile,
provider or fixed-point proof is accepted; I3–I5 and V1–V3 remain unchecked.

## Successor resolution handoff (2026-11-06)

The consuming-workspace lock/patch repair has failing-before/passing-after
behavioral regressions and focused Clippy/build evidence in
[successor resolution](evidence/2026-11-06-successor-resolution.md). Real
preparation in Pueue 31 passed the stale `memchr 2.7.1` selection and reached
the fixed target's unclassified `target_has_atomic = "64"` predicate. Source
pins and resource bounds were unchanged. No prepared profile or cold proof is
accepted; I3–I5 and V1–V3 remain unchecked.

## Successor admission checkpoint (2026-11-06)

Consuming lock/patch scope, fixed native cfg selection, Cargo-feature namespace
separation and lossless producer labels now have final focused execution evidence
in [successor resolution](evidence/2026-11-06-successor-resolution.md).
Pueue 86 passed 2 label-admission and 15 successor-resolution tests, focused
release Clippy and CLI build. Real preparation in Pueue 88 passed the preceding
admission boundaries and now stops at `source recipe capacity exhausted before
original LLVM graph`. Existing bounds, reservations and profile pins were not
weakened. No prepared profile or provider/fixed-point acceptance is implied;
I3–I5 and V1–V3 remain unchecked.

## Compact native continuation checkpoint (2026-11-06)

The shared controller now retains native script reservations at runtime through
eleven fixed declarations. Protected request/replay, native nonzero results,
exhaustion, exact images, immutable history and fatal-status boundaries have
executed evidence in [compact native continuations](evidence/2026-11-06-compact-native-continuations.md).
Macro/helper regressions, actual completion-utility execution, final focused
Clippy and the release CLI build passed. Real preparation in Pueue 31 passed the
prior pre-LLVM recipe-capacity boundary and now rejects `ambiguous original
locked identity home` in successor resolution. Pins, source reservations and
resource ceilings were unchanged. No prepared profile, provider execution or
fixed-point proof is accepted; I3–I5 and V1–V3 remain unchecked.

## Source-qualified lock checkpoint (2026-11-06)

Exact source qualifiers and borrowed owner identities now remain bound through
transitive resolution and feature revisits. Original Cargo decoder inspection,
offline host metadata, failing-before regressions and 22 passing focused tests
are retained in [source-qualified lock resolution](evidence/2026-11-06-source-qualified-lock-resolution.md).
Focused release Clippy and CLI build passed. Unchanged-pin real preparation in
Pueue 38 passed the `home` ambiguity and now rejects the typed relative operand
`x86_64-linux-musl/lib/`. Subsequent attribution found an empty-filename join,
not an original directory suffix; normalized path admission remains strict.
No prepared profile, provider or fixed-point proof is accepted; I3–I5 and
V1–V3 remain unchecked.

## CRT operand checkpoint (2026-11-06)

The native SDK author now binds the CRT directory directly for an empty library
filename, matching the existing mrustc SDK convention. Pueue 53 passed all 27
recipe tests, leaf rustfmt, focused strict release Clippy and the release build.
Unchanged-pin real preparation in Pueue 54 passed the malformed relative member
and now rejects the LLVM original-project command's missing bounded stdout.
Logs and source attribution are retained under
[CRT directory operand correction](evidence/2026-11-06-source-qualified-lock-resolution.md#crt-directory-operand-correction).
No prepared profile, provider execution or fixed-point proof is accepted;
I3–I5 and V1–V3 remain unchecked.

## Source program obligation checkpoint (2026-11-08)

Bounded command streams, whole-program action obligations, canonical chunk
transport for oversized authored members, and staged Cargo target sources now
have focused execution evidence in
[source program obligations](evidence/2026-11-08-source-program-obligations.md).
The staged-source repair required refreshing only the Mantle source record from
the same frozen revision; the refreshed profile identity is
`e2f0191b78a6715e7c12394166aae1fc07b0fa6b79a9567cb2d4d54754ce5020`, which changes
the reviewed source-profile pin and is not adopted here. With that profile,
preparation hydrated the complete source, authored both program sets, and
reached the Cargo-free plan for Mantle itself, where the Cargo-free topology gate
blocked. Test-only registry requirements no longer resolve into build-mode facts,
and a git edge version is now a requirement rather than an identity, so the
captured `turso` fact binds. Repeating the probe leaves exactly two blockers,
both intra-repository path dependencies inside vendored git sources
(`turso_core`'s `core/` sibling and `basalt`'s generated `vendor/ucan`): per-crate
vendoring does not preserve repository layout. Resolving that is an
input-cohort decision, not a planner repair. No prepared profile, provider
execution or fixed-point proof is accepted; I3–I5 and V1–V3 remain unchecked.

## Captured git repository checkpoint (2026-11-08)

The operator chose captured repository trees and adopted the refreshed
source-profile identity. Git-sourced packages now resolve inside
`<vendor root>/git-repositories/<host>/<path>`
([ADR 0146](../../adr/0146-resolve-git-packages-inside-captured-repository-trees.md));
`scripts/capture-git-repositories.py` captured 11 repositories at their locked
revisions, and `mantle source bundle refresh-vendor-inputs` replaced only the
vendor record of profile `e2f0191b…`, producing `a6174213…`. Focused tests,
leaf formatting, strict release Clippy and the release build passed. The two
intra-repository path-dependency blockers are gone; the package/target layer now
names seven concrete blockers. Additive member features, inherited optionality
and workspace-root-relative path dependencies are now admitted per Cargo
semantics, unset bare cfg flags and `target_feature` predicates evaluate false as
Cargo does, a member `default-features` value overrides the inherited one, and
only planning-workspace members keep strict dev material. All four Cargo-free
planning layers then report ready. Real preparation advanced past planning and
through four authoring boundaries: plan-derived source operands are now
normalized lexically before admitted-root stripping, and a metadata frame's own
`script`/`package`/`metadata-helper`/`request`/`rustc` bindings supersede
inherited compiler inputs instead of colliding. `--extern` producers are now
resolved across host-artifact and registry-artifact spellings, crate-name and
target-name spellings, host-dependency role and build-script units. Preparation
now stops at a workspace path package with two non-host producers
(`crunch-rustc-wrapper@0.1.0`), whose plan entry carries no producer link. That
is repaired by mirroring the planner's own first-target-lib selection, and the
authored fetch record now uses the bundle's store-prefix rule.

## Preparation completion checkpoint (2026-11-09)

Authenticated source-program preparation now succeeds: Pueue 29 exited 0 and
produced a cohort binding `input_profile_blake3 a6174213…` and
`output_profile_blake3 e5f4f916a1e07d5aa25f4ca53c609f7ab424adbb5cfd4a3907895bb283e56022`,
with four inert program records. The report keeps its own non-claim. Details,
identities and the cleared boundaries are retained in
[preparation completes](evidence/2026-11-09-preparation-completes.md). A prepared
cohort is not an accepted profile, a provider, a stage comparison, a fixed point
or release eligibility; I3–I5 and V1–V3 remain unchecked and the cohort still
needs independent review before a proof launch pins it.

## Cold proof launch checkpoint (2026-11-09)

The prepared cohort and the current CLI were transferred to Leviathan with exact
post-transfer parity (`mantle e8770dbe…`, profile `6ef987ad…`, report
`bd9afba3…`), and Pueue task `1514` launched the promoted cold proof there with
`PREPARED_PROFILE_BLAKE3 e5f4f916…`, the unchanged StageX/native pins, explicit
bwrap/static-shell paths and a 200 GB disk bound. Launch inputs, host facts and
non-claims are retained in
[cold proof launch](evidence/2026-11-09-cold-proof-launch.md). A launched proof
is not a proof: stage equality, action reconciliation, the v2 receipt, release
evidence and independent review remain outstanding, and I3–I5 and V1–V3 stay
unchecked until that evidence exists.

The first launch failed closed in ten minutes: the authored cold route named one
role (`native-source-bundle`) for selectors whose consumers authenticate distinct
roles. Preparation now places the StageX program in the StageX source-bundle
closure, the Rust program beside the Rust source archives, and names
`stagex-source-bundle` / `rust-source-archive-set` in the route. A second cohort
was prepared (`output_profile_blake3
10d75988e3f285885a6aa5243c7b205b1150bf92ab02f7d2d6948b2fd71050e4`,
profile BLAKE3 `b98816e9…`) and launched on Leviathan as Pueue `1515`. Details in
[program placement and second cohort](evidence/2026-11-09-program-placement-and-second-cohort.md).
The second launch then rejected a StageX runtime mount (`/mantle/stagex/bison-data`)
outside the admitted namespaces; the generator now derives that destination from
the plan's logical store prefix. A third cohort
(`output_profile_blake3 82b8a7ca127af82c7c8718e2ad60728f13db7a6a15d86d226fec64bcad951703`,
profile BLAKE3 `db03fdf0…`) was prepared and launched on Leviathan as Pueue
`1516`. Stage equality, action reconciliation, the v2 receipt, release evidence
and independent review remain outstanding; I3–I5 and V1–V3 stay unchecked. Details are retained in
[captured git repositories](evidence/2026-11-08-captured-git-repositories.md).
No prepared profile, provider execution or fixed-point proof is accepted;
I3–I5 and V1–V3 remain unchecked.

The first promoted run then exposed three authoring boundaries, each now fixed:
the canonical StageX closure must keep the manifest identity its lineage pins
(authored bytes live in the native closure instead), lineage-declared inputs must
be consumed (the empty after-hook and the StageX patch source are consumed as
standard input), and StageX runtime mounts derive from the plan's logical store
prefix. A seventh cohort
(`output_profile_blake3 205503813fb8256f02c07fd59c93579965f12340bfbf5a373d3f88204b4140cc`)
ran for 1 h 41 m before the partition validator reported an omitted lineage
dependency; the validator now enumerates every omission so the next run yields
the complete list. Details are retained in
[authored placement and seventh cohort](evidence/2026-11-09-authored-placement-and-seventh-cohort.md).
No stage equality, action reconciliation, v2 receipt or release evidence exists;
I3–I5 and V1–V3 remain unchecked.

The enumerated run (Pueue `1522`) then reported all eleven omitted lineage
dependencies at once. They split into replaced script-interpreter images
(`kaem-0`, `stage0-kaem`), which need an explicit replaced-input declaration in
the authored index rather than a silent omission, and apparent authoring gaps in
the mes/tinycc linking leaves (`mescc-entrypoint`, `mes-crt1`, `mes-libc-tcc`,
`tcc-boot0`, `tcc-mes`, `tinycc-crt1`, `tinycc-libtcc1`), which need the original
operands. That audit is the next slice; no stage equality, reconciliation, v2
receipt or release evidence exists, and I3–I5 and V1–V3 remain unchecked.

## Native probe state checkpoint (2026-11-13)

Proof v27 rejected ordinary file authority from potentially failing native
probes. Driver-result and skipped-assembler authoring now retain the prior
admitted file state while preserving real command status and stdio. The
positive/negative graph-admission regression, focused strict release Clippy,
release build, current source-built tests, and CLI qualification passed. A new
cohort and cold proof remain unaccepted. See
[native probe state authority](evidence/2026-11-12-source-input-binding-corrections.md#native-probe-state-authority-2026-11-13).
I3–I5 and V1–V3 remain unchecked; the preserved v27 failure is not a proof.

## Owning-stage and canonical Rust handoff checkpoint (2026-11-13)

Independent review rejected the inert `47c446…` cohort for cold launch: shared
translation/link references named the Rust stage inside Mantle stage1/stage2,
and the Rust provider published `tree` rather than the strict canonical output.
Source `4e0f7ad0f228773710e57ea8938e1d70d73fdc20` fixes both without relaxing
admission. Leviathan Pueue 1548 passed 255 source-built tests (27 protected smokes
ignored), package formatting, strict release Clippy and a locked offline release
build, retaining equal pre/post frozen source identities. Details and exact
logs are in
[source input corrections](evidence/2026-11-12-source-input-binding-corrections.md#owning-stage-and-canonical-publication-correction-cohort).

Tasks 1549 and 1550 respectively prepare the replacement inert cohort and run
isolated canonical CLI/contract/lifecycle gates. Neither submission accepts a
profile, provider, stage comparison, receipt or release. The rejected cohort's
cold-proof script was not executed. I3–I5 and real V2/V3 evidence remain open.

## Nested native-script stage checkpoint (2026-11-13)

The replacement `4932432f…` profile passed source identity verification, but
independent graph review rejected 740 remaining Rust-stage input references in
each Mantle recipe. Shared LLVM `text`/`assemble` products now derive the owning
recipe stage too. Task 1551 passed the whole native-script graph-admission
regression for Rust, Mantle stage1 and Mantle stage2 with static/captured
environment/cwd inputs and a rejected foreign-stage producer. The actual cohort
must still be regenerated and independently admitted; its prepared cold-proof
script was never run. I3–I5 and proof-dependent V2/V3 remain open. See the same
source-input correction evidence for both complete reviews and the focused run.

## Selected topology and authenticated PerlAsm checkpoint (2026-11-13)

The `11498fc8…` cohort's stage references are valid, but graph review rejected
1,975 unreachable Mantle steps; semantic roots account for all but 12 unconsumed
PerlAsm authentication results. Semantic products now remain execution roots,
and PerlAsm reads a typed authenticated file/member while preserving logical
pathname and AWS's source transformation. Strict reachability and input pruning
are unchanged. Task 1556 passed five graph regressions plus actual bounded host
ring/AWS helper and Perl-wrapper fixtures; the exact results and failed fixture
compile from task 1555 are retained in the source-input correction evidence.
ADR 0149 records ownership and the clean internal contract cutover. No cold
proof launched from this graph-rejected cohort; I3–I5 and proof-dependent V2/V3
remain open pending a new frozen cohort and actual execution.

## Source-ready semantic-closure checkpoint (2026-11-14)

The source-bound `22eb45ca…` runner completed inert preparation in task 1560.
Both actual bundle verifiers reported ready; exact program/report identities,
the interrupted predecessor and cross-endpoint profile-byte parity are retained
in `evidence/2026-11-13-leviathan-22eb45ca/`. Independent complete-graph and
source-identity/PerlAsm-consumption reviews gate cold launch. Preparation and
transfer do not complete I3–I5 or proof-dependent V2/V3. See the source-input
correction evidence for the complete current handoff.


## Original AWS source intake checkpoint (2026-11-14)

The `23c026be…` inert cohort passed complete graph and guard inspection but is
launch-rejected because its required original AWS-LC tree is absent. Task 1561
successfully acquired the exact declared upstream Git commit/tree; captured
identities and provenance are in `evidence/2026-11-14-aws-original/`. The existing
cohort is not modified or launched. A new complete snapshot must include the
guarded original PerlAsm pipe repair as well as the acquired source before
qualified runner build, profile regeneration and source-completeness review.
The source-complete `22eb45ca…` assembly is diagnostic only. I3–I5 and V2/V3
stay open.


## Guarded original PerlAsm checkpoint (2026-11-14)

The original pipe adapter now consumes either exact supported upstream statement
form, including the complete guarded continuation, without modifying its input
or replacing the generator algorithm. Pueue 1568 passed the focused behavioral
tests, package formatting and strict release Clippy. Pueue 1569 compared every
selected Ring/AWS-LC invocation against its original pipe with byte-identical
assembly. Exact results, prerequisite failures and bounded fixture non-claims
are in the source-input correction evidence and `evidence/2026-11-14-pipe-boundary/`.
This completes the pipe repair only; a corrected source-complete frozen runner,
regenerated cohort and independent review are still required. I3–I5 and real
V2/V3 remain open.


## Original-source-complete preparation checkpoint (2026-11-14)

Frozen code `8fa84260…` plus the exact original AWS-LC tree passed source-bound
runner qualification/build in task 1571 and the canonical CLI/contract/lifecycle
gates in 1572. Task 1573 successfully prepared a fresh immutable `1ce22ef7…`
cohort; both actual source verifiers reported ready and complete profile-byte
transfer parity passed. Exact source/program/runner identities and transcripts
are in `evidence/2026-11-14-leviathan-8fa84260-aws/` and the source-input
correction evidence. Independent full graph/guard and physical source/program
reviews gate its first cold launch. No provider execution, stage comparison,
reconciliation, receipt or real parity/release acceptance follows from this
preparation. I3–I5 and proof-dependent V2/V3 remain open.

## Reviewed original-source-complete cold attempt (2026-11-14)

Current independent graph/guard and physical source/program checks now support
launch of the `1ce22ef7…` prepared cohort. Both original trees match the pinned
Git files/modes, selected authenticated source routing and stage-local selection
parity passed, and complete source/profile transfer parity is retained. Luna's
quota failure is preserved without inventing a final model verdict; actual
completed inspections and the integration owner's remaining static checks are
bound in `evidence/2026-11-14-leviathan-8fa84260-aws/launch-review-acceptance.json`.
Pueue 1574 is the exact cold proof command from frozen code `8fa84260…`, with
unchanged independent pins and strict bounds and no development/remote-action
route. I3–I5 and proof-dependent V2/V3 remain unaccepted pending real execution,
reconciliation, receipt, parity and release evidence.

## Typed member and indexed admission checkpoint (2026-11-15)

Cold task 1574 failed closed while preparing `unit-00251-script-original-package`:
its generated-member exclusion was an untyped literal filesystem path. No
provider, stage, receipt or successful-proof alias was accepted. The author now
uses a typed member of the original package, and the real helper rejects
relative, foreign-tree, root and traversal exclusions without changing the
original source tree.

Dependency admission now indexes immutable recipe steps and actual operands
once per validation pass, preserving exact producer namespaces, guard/seed
dependencies, both observed-phase state alternatives and ownership checks.
Pueue 1579 passed 263 source-built tests (27 ignored), formatting, strict focused
Clippy and a release build against the byte-bound four-file overlay. The source
tree digest was unchanged through qualification. Inputs, commands, complete
output and failed-attempt boundaries are retained under
`evidence/2026-11-15-typed-member-index/`.

The separate native row construction reached a real missing prerequisite:
the reserved StageX transition can only be adopted after current protected
execution, not imported from historical outputs or cache hits. Existing native
row receipts remain unchanged. A new committed source snapshot, qualified
runner, regenerated cohort and current launch review are still required;
I3–I5 and proof-dependent V2/V3 remain open.

## Committed typed-member source checkpoint (2026-11-15)

Commit `e9a58881…` is pushed. Complete-source assembly 1581, source-bound
qualification/build 1582 and canonical CLI/contract/lifecycle gates 1583 all
completed successfully. The frozen complete source identity is `e62509cd…`;
263 source-built tests passed with 27 ignored, and the runner remains byte-bound
through unchanged pre/post source checks. Exact byte-verified logs and source
metadata are in `evidence/2026-11-15-leviathan-e9a58881/`.

Preparation 1584 is gated on both successful qualifications and must author a
new cohort. No prepared profile, protected provider, Mantle stage, receipt or
fixed point is accepted from these checks; physical parity, current independent
launch review and actual execution remain required. I3–I5 and proof-dependent
V2/V3 stay open.


## Regenerated committed cohort checkpoint (2026-11-15)

Task 1584 completed source-only preparation for frozen `e9a58881…`. Both actual
bundle verifiers report ready, and the new prepared manifest is `30960523…`.
Current program/report identities and full preparation output are retained in
`evidence/2026-11-15-leviathan-e9a58881/`. Full-profile byte-stream transfer
parity and the requested complete current Luna inspections gate the unlaunched
cold command. This completes regeneration only, not protected provider/stage
execution, a proof receipt, real native parity or release acceptance. I3–I5 and
proof-dependent V2/V3 stay open.

## Deferred invocation authoring checkpoint (2026-11-15)

The e9a58881/30960523 cohort remains unlaunched and is held on a real deferred
request mismatch. The complete inert word scan found 2,943 flags inside
consumed Rust invocation JSON, despite all direct recipe words passing its
lexical/shape checks. Compiler-data strings were emitted as filesystem Literals,
and source target paths retained manifest dot/parent components. Actual request
admission validates those words unchanged before the executable can run.

Local Pueue 26 executed the new real-profile/decoder regression against isolated
pre-fix sources and failed with the exact untyped filesystem-literal error.
Leviathan 1585 executed and passed that repaired profile regression and the
existing path/name/control authority rejection test (one test per leg). Its
mistyped resolver filter selected zero tests and is not resolver evidence;
correct module/boundary verification in 1586 then executed 23 tests and passed,
including sibling/dot source members and rejected archive escapes/absolute paths.

The author-only repair keeps cfg/rpath bytes as LiteralData and resolves source
members with the existing archive-relative resolver. The consumer and strict
validator are unchanged. Evidence and exact commands are in
`evidence/2026-11-15-leviathan-e9a58881/parent-word-inspection/`.
Final committed-source qualification, regeneration, complete current byte and
Luna reviews, protected stages, reconciliation and v2 receipt remain required;
I3–I5 and proof-dependent V2/V3 stay open.

## Invocation source qualification checkpoint (2026-11-15)

Author repair `8d2dfcd8…` is pushed. Assembly 1587 succeeded, but source
qualification 1588 stopped on one rustfmt wrap. Canonical 1589 executed its
named test legs successfully, then rejected the new profile regression module's
missing private JSON-family decision; the following Cairn gates did not run.
Preparation 1590 was dependency-failed and never ran. The exact wrap is applied,
and the new fixture is registered in the existing internal bootstrap family.
Corrective task 1591 passed leaf rustfmt and the machine-contract checker
(`24 contracted, 65 classified`). Commands, full logs and bounded outcomes are
in `evidence/2026-11-15-leviathan-8d2dfcd8/qualification-corrections.md`.
A newly committed exact source must still qualify and regenerate; I3–I5 and
proof-dependent V2/V3 remain open, with no new cohort or protected-stage claim.

## Frozen runner artifact binding hold (2026-11-15)

For published `71c2bd4d…`, assembly 1592, source/test/Clippy/build commands 1593
and canonical CLI/contract/Cairn commands 1594 completed successfully. The
source leg executed 265 passed and 27 ignored. However, the installed regular
runner retained the rejected e9 runner's byte identity; its shared release
artifact predates this qualification while frozen source mtimes are epoch.
Command/source parity therefore does not establish fresh regular-runner binding.

Own source-only preparation 1595 was stopped and retained, with no accepted
cohort or protected execution. Fresh-target source and canonical jobs 1596/1597
use separate absent-before-start directories and unset compiler wrappers inside
the checked-in shell. Replacement preparation 1598 is stashed pending both
actual results and parent runner-byte/source inspection. ADR 0150 records this
operator qualification boundary; ordinary developer caches are unchanged.
Exact observations and non-claims are in
`evidence/2026-11-15-leviathan-71c2bd4d/runner-binding-hold.md`.
I3–I5 and proof-dependent V2/V3 remain open.

## Actual fresh runner checkpoint (2026-11-15)

Fresh-target source qualification 1596 and canonical 1597 completed Success
for frozen `71c2bd4d…`. The real regular binary was compiled and installed as
`mantle-71c2bd4d-fresh-target`, BLAKE3 `c9803076…`, distinct from the stale e9
image. The complete source identity stayed `b49c087a…`; source execution was
265 passed, 27 ignored, and the isolated canonical CLI/contract/Cairn sequence
completed. Exact full logs, byte parity and source/runner bindings are in
`evidence/2026-11-15-leviathan-71c2bd4d/fresh-runner-qualification.md`.

Only after parent inspection was source-only preparation 1598 enqueued from its
stash into a new absent cohort namespace. No ready cohort, full current review,
protected provider, Mantle stage, reconciliation, fixed point or release
acceptance follows from runner compilation. I3–I5 and proof-dependent V2/V3
remain open.

## Fresh-runner cohort prepared (2026-11-15)

Source-only preparation 1598 completed Success using the genuinely compiled
`c9803076…` runner. Both actual source verifiers report ready with empty
missing/stale/unsupported/untrusted sets. The new input profile is `6e3a630f…`,
input plan `1e74d77c…`, and prepared manifest `c22381a8…`. The complete frozen
source remained `b49c087a…`; the original input profile also remained unchanged.
Exact task, commands, report, four new program identities, source verifier
outputs and received-metadata parity are retained in
`evidence/2026-11-15-leviathan-71c2bd4d/fresh-preparation-completed.json`.

The complete prepared profile is 15,791,130,195 bytes. Its raw BLAKE3 is not its
manifest identity. Complete-profile transfer parity and fresh independent Luna
graph/guard/typed-template and physical/authenticated-source reviews are now
in progress under the current `review-contract.md`. The old e9 cohort,
interrupted 1595, prior model verdicts and historical native rows are not
substituted. No protected execution has been launched. I3–I5 and proof-dependent
V2/V3 remain open.

## Current cohort held on confirmed source admission defects (2026-11-15)

Complete profile transfer matched raw BLAKE3 `54f27df5…` and its source-only
evidence was pushed as `13b8d72a`. The current cohort is nevertheless not
launchable: both Mantle stages omit the NativeSourceBundle authority used by
three exact helper ManifestSource members per stage. The unchanged consumer
rejects all six. The shared stage declaration is repaired without changing
source pins or recipe validation; actual source-built execution in task 1602
reported `266 passed; 0 failed; 27 ignored`, including real manifest-member
lowering and rejection after removing permission even with a populated cache.
Strict release Clippy also completed. Exact commands/results are retained in
`evidence/2026-11-15-leviathan-71c2bd4d/helper-source-authority-dev*`.

The graph review separately confirms an invalid `CARGO_FEATURE_DEP:CORE`
environment operand in a consumed Rust invocation. Dependency activation must
not be emitted as a Cargo feature; its authoring repair remains open. Other
raw graph diagnostics include inspector-context errors and incomplete nested
controller coverage, not thousands of independently established product bugs.
The incomplete Luna identity inspector is being repaired and executed by the
parent; its cancelled/failed attempts have no accepted physical verdict.

No protected launch occurred. Preserve the current frozen source/cohort and
review failures. Product changes require newly frozen/qualified source and
runner, source-only regeneration and complete current review. I3–I5 and
proof-dependent V2/V3 remain open.

## Both confirmed author/declaration repairs verified (2026-11-15)

Task 1603 executed the actual source-built leg with `267 passed; 0 failed;
27 ignored` and strict release Clippy. It includes both Mantle stages' genuine
helper manifest admission/cached-authority rejection, and resolver selection
that preserves an activated optional dependency while excluding `dep:` tokens
from published features and keeping disabled dependencies absent. Local changed
code BLAKE3 values equal the formatted remote files actually exercised.

The minicargo boundary now matches the existing successor convention: dependency
selectors remain available for optional-edge activation but do not become cfg
or Cargo feature-environment names. Exact command/result scope is retained in
`evidence/2026-11-15-leviathan-71c2bd4d/helper-and-feature-authority-dev*`.
These ordinary development checks do not bind a replacement frozen runner.

The old `c22381a8…` cohort remains held and untouched. Parent physical v7
completed all seven role payload comparisons before an inspector confused
chunk entries with physical files; the corrected audit retains every byte,
chunk, archive and source-role obligation. Static nested-controller review
continues. Freeze/publish the verified product changes, qualify a genuinely
fresh-target runner and regenerate/review a new cohort before any launch.
I3–I5 and proof-dependent V2/V3 remain open.

## Corrected frozen runner accepted; regeneration released (2026-11-15)

Both product repairs are published as `787784fb`. Complete frozen source
`dcfcf2de…` qualified in two separately extracted trees and previously absent
Cargo targets: runner task 1606 completed Success with `267 passed; 0 failed;
27 ignored`, formatting, strict Clippy and release build; canonical task 1607
completed the selected CLI, receipt, closure, machine-contract, release-core
and Cairn gates. Both wrappers recorded unchanged source identities. Independent
installed/fresh-target binary measurements equal `6b7c8409…`. Full source,
task, log-transfer and byte bindings are retained in
`evidence/2026-11-15-leviathan-787784fb/fresh-runner-qualified.json` and
`fresh-runner-qualification.md` beside it.

Only after accepting those actual results did the parent release the existing
stashed source-only preparation 1608. Its new report, source verifiers, plan,
manifest, raw-profile transport parity and complete new-subject reviews still
must be observed; no prior hashes/counts/verdicts are copied forward. The exact
release is retained in that evidence directory's `preparation-released.json`,
and `review-contract.md` defines current graph/physical review obligations.

Old-cohort parent physical v11 reached all seven role byte comparisons, then
erroneously required the excluded `.autoresearch` checkout entry. The frozen
consumer stages an explicit build-input allowlist, excludes the separate
vendor role and `.git`, and records leaves rather than empty directories.
Corrected physical v12 compares every admitted frozen leaf and required parent,
records policy hashes/excluded roots/additions, and retains all existing
archive, chunk, source and program identity checks. The failed inspector output
is preserved; no product admission change follows from that failure. Complete
old graph review remains distinct from reviews of the replacement subject.

The rejected `c22381a8…` cohort remains unlaunched. Runner qualification and
source preparation are not source-built compiler/provider execution, a fixed
point, release eligibility or signed native-row acceptance. I3–I5 and
proof-dependent V2/V3 remain open.


Parent physical v12 subsequently completed with bounded inert ACCEPT for the
old `c22381a8…` subject. The transferred result verifies all seven role payloads,
59 original native record bodies, four new program records, 26 selected AWS-LC
and 12 Ring scripts, and preservation of all 12,762 admitted frozen source
leaves. The 11,626 policy-excluded leaves and four additional members are
separate coverage, not a whole-checkout or preauthor/postauthor identity claim.
Exact result, commands, policy hashes and transfer equality are in
`evidence/2026-11-15-leviathan-71c2bd4d/identity-review/parent-v12-completed.json`
and its referenced result. This physical verdict does not repair the old
consumer rejections, certify replacement preparation 1608 or grant launch
authority. Complete graph coverage remains separately required.


## Corrected cohort regenerated and transport-bound (2026-11-15)

Source-only preparation 1608 completed Success from qualified product
`787784fb`, runner `6b7c8409…` and unchanged frozen source `dcfcf2de…`. Both
actual source verifiers report ready with empty missing/stale/unsupported/
untrusted sets. The new input manifest is `6fab5e96…`, input plan `2c77de53…`
and prepared manifest `9300895d…`; actual metadata cross-links and transferred
metadata bytes agree. All four program identities and the complete seven-role,
six-stage source/authority/receipt/resource contracts are retained in
`evidence/2026-11-15-leviathan-787784fb/fresh-preparation-completed.json` and its
referenced current report/plan files.

The entire 15,790,233,347-byte prepared-profile SSH stream matches independently
measured raw-file BLAKE3 `fbc5c0e4…`. This is not the canonical manifest identity.
`profile-stream-completed.json` records full transport parity and
`review-subject.json` binds exact current review inputs in the same directory.

Only after those actual publication/parity results were accepted did the
parent dispatch `CorrectedGraphLuna` and `CorrectedIdentityLuna` together using
the requested exact model. They own disjoint graph/template and physical/
authenticated-source slices under the updated `review-contract.md`, with
frozen data read-only and no product/helper/controller/provider execution or
proof launch. Prior subjects, raw hashes, measured counts and verdicts are not
adopted. Their complete current reports still require parent integration.

Qualified source-only regeneration is complete, not a protected compiler/
provider build, Mantle stage, reconciled v2 receipt, fixed point, release or
signed native-row acceptance. I3–I5 and proof-dependent V2/V3 remain open.


## Current GCC specs consumer rejection reproduced (2026-11-15)

The completed baseline graph review and the independent current-subject review
confirm that `generator-base.relocated-smoke.specs` consumes an authenticated
request naming `unit.c` and `target` without declaring either on that Exec.
The unchanged decoder rejects it even though this step is outside dependency
reachability, because every recipe step is scheduled. The `9300895d…` cohort
therefore remains read-only and unlaunched; availability/transport success is
not consumer admission.

Task 1612 executed the real new decoder regression against unchanged production
code and reported `0 passed; 1 failed`, with `source recipe: unknown input unit.c`.
Its complete output and exact source/task bindings are retained under
`evidence/2026-11-15-gcc-specs-request-inputs/`. The local shared author now takes
an explicit source-owned request read set, retains those typed dependencies,
and migrates both authenticated-request callers without modifying request
bytes or relaxing decoder checks. The regression also requires rejection after
removing either addressed binding or adding an unused declaration; incidental
map-copy tests are removed. ADR 0153 records the contract and owner.

The first fixed verification stopped at a compiler lifetime error before tests;
that failure is retained. The corrected source is under actual source-built
and strict-Clippy verification in task 1614. No successful result, replacement
runner/cohort, protected execution or v2 acceptance is inferred while it runs.
Remaining current graph/template and physical findings still require complete
review and parent integration. I3–I5 and proof-dependent V2/V3 remain open.


Task 1614 subsequently completed Success: the real decoder regression and the
source-built leg reported `266 passed; 0 failed; 27 ignored`, followed by strict
release Clippy. The count replaces two incidental name/map-copy tests with the
actual producer-to-decoder authority regression. Local source BLAKE3 values
match the exact formatted files exercised; the full transcript, task and source
bindings are retained in that repair's evidence directory. This verifies the
GCC author repair, not a final frozen runner or replacement cohort.

The same current graph review independently confirmed 57 Rust translation
`OUT_DIR` mismatches in `9300895d…`: each operand names a descendant `/tmp/.../out`
Workspace mount while its invocation declares only `/tmp`. The frozen consumer
requires exact map ownership, so these are actual static decode rejections, not
inspector-context errors. Parent owns a captured-tmp author repair that uses the
declared `/tmp` root plus validated relative member while preserving the original
virtual pathname. Overlap, unknown-root and traversal admission remain strict.
ADR 0154 records the boundary; the real emitted-invocation decoder reproduction
is running in task 1615, before the changed author is staged into its snapshot.
No successful workspace verification or corrected cohort is inferred yet.


Task 1615 completed with Failed(101): the actual emitted invocation reached
`decode_invocation` and failed with `workspace word must name an exact invocation
workspace mount`; the execution reported `1 passed; 1 failed; 0 ignored`.
`evidence/2026-11-15-captured-tmp-workspaces/` retains the command, complete log
and task result. Only after that snapshot finished was the root/member author
repair staged. Fixed verification task 1616 runs the affected source-built leg,
leaf formatting and strict release Clippy; no success is inferred while it runs.


Task 1616 completed Success. The actual repaired decoder and source-built release
leg reported `268 passed; 0 failed; 27 ignored`; strict release Clippy completed
successfully. All five changed Rust files match the exact formatted remote bytes
exercised. `evidence/2026-11-15-captured-tmp-workspaces/` retains the complete log,
task result and source bindings. The two author repairs are locally verified;
they do not update the rejected frozen `9300895d…` programs.

The current graph reviewer returned bounded REJECT, not complete graph coverage.
It explicitly left nested catalogs, all invocation words, guards/ObservePhase
dominance, source-node closure, all helper authority and semantic/smoke roots
without exhaustive total/covered counts. Parent does not accept that report as
launch permission and requested retention of its actual commands/results. A
replacement published/frozen/qualified source, regenerated programs, and complete
current-subject graph and physical reviews remain required. No protected proof
was launched from `9300895d…`; I3–I5 and proof-dependent V2/V3 remain open.


## Published request/workspace repair qualification (2026-11-15)

Verified author repairs and their actual failed/successful executions were
published as `869a461d6170404e752865a2029092391240b741`. New archive transfer
matches at compressed BLAKE3 `6ce9eb923e01a36f967a4d07cf1d15caf751f2f906048aaeb3faf8a910217449`.
The first tmpfs staging attempt failed with no space; its partial owned archive
was removed and compressed-only disk-backed staging completed. The failure is
retained, not claimed as a completed archive.

Complete source assembly 1618 succeeded. New normalized source identity is
`f1c0468c177aa6ccc0cfed3ea950726123f9528cce0b9e6b7c18229e607fa911`.
Canonical qualification 1620 succeeded in its own initially absent target, with
that exact source identity unchanged before/after all recorded CLI/receipt/
closure/release/machine-contract and Cairn gates. Full commands, task results
and each actual test result are retained under `evidence/2026-11-15-leviathan-869a461d/`.
Isolated runner qualification 1619 still requires its actual result; preparation
1621 is stashed until parent accepts both qualifications and runner/source bytes.

Parent is preparing complete explicit-subject graph/template/physical inspectors
rather than accepting partial model text or old hardcoded counts/verdicts.
ADR 0155 records complete per-criterion coverage, actual scopes/kind provenance,
and separate symbolic runtime versus observed execution boundaries. New-data
execution/acceptance is not inferred from implementing these inspectors.
No protected attempt is selected from either rejected old data or this pending
replacement; I3–I5 and proof-dependent V2/V3 remain open.


## Replacement source-program publication observed (2026-11-16)

Source-only preparation task 1621 completed Success from the qualified
`869a461d…` source and `ef890d46…` runner. Both source verifiers reported ready,
with empty missing/stale/unsupported/untrusted sets. Frozen source remained
`f1c0468c…`; the input profile stayed byte-identical before/after generation.
The actual input manifest is `2ac7c54bb85544ea738c04c2bc073e576984b2d661dfb0aaf58da24f1e440cf2`,
input plan `670b518868884548d6570cdf9ed050086f6184c4bc4c7a95b17fdc8a9e78d0cd`,
and prepared manifest `79f9c65041788459714b26dd7dccc3dd6463e864a0b2d4feca703b09760da2ce`.

The complete 15,795,883,182-byte profile stream over SSH matches the actual raw
file BLAKE3 `9d26367b89e60c7b980339ddb4da9400d15c3487720dcdee8e57220680f7c442`.
These raw bytes are not the canonical manifest identity. Actual metadata,
source-verifier results and stream output are retained under
`evidence/2026-11-15-leviathan-869a461d/`; no old counts or program identities
are carried forward. Publication/freeze/qualification/regeneration is complete.

Both newly requested `underclass/gpt-6-luna` physical-inspector workers failed
with `429 ... quota_exhausted`. No other model was substituted. Parent is
completing the reachable checker integration and actual current-data exercise;
those parent checks are not independent exact-model acceptance. The partial
failed-worker edit was repaired in inspection code only; frozen source and
generated programs were not modified.

Current full graph/template inspection task 1623 settled, but its actual result
has not yet been retrieved. Current full physical inspection 1624 and streaming
preauthor role reconstruction 1625 were submitted against the measured new
subject. Retrieving the settled static result then encountered SSH connection
timeout; no success is inferred from the wait command. Exact current verdicts,
complete per-criterion coverage and independent-review authority remain required.
No protected proof has been launched from this replacement.

## Complete current physical checks observed (2026-11-16)

Parent-owned task 1624 completed Success and emitted
`STATIC_IDENTITY_CHECKS_CLEAR`: all seven source roles, all four authored
program records, all 59 original native record bodies, 476 archive members,
20,854 original Git leaves and 27 initially absent authority paths were covered.
Task 1625 independently exercised the preauthor reconstruction mechanism and
completed Success with `STATIC_PREAUTHOR_IDENTITIES_CLEAR`. All seven role
criteria and every expected leaf record were covered. The Mantle, Rust archive
and vendor directory identities reconstructed from actual embedded input bytes
match the preauthor plan; bootstrap-source input bytes, the assembled native
plan closure and the augmented native closure remain distinct identities.

Complete received reports match the remote BLAKE3 values recorded in
`evidence/2026-11-15-leviathan-869a461d/identity-review/physical-result-transport.json`.
These bounded physical results close the earlier missing preauthor-directory
measurement; they are not independent exact-model acceptance or execution.

Static task 1623 failed in inspector aggregation; its failed output and original
source remain retained. Corrected task 1626 produced an actual graph `HOLD`,
not acceptance. Its template subprocess had a missing `__main__` entry point:
process Success without a result was rejected. Task 1627 invokes that entry
point and requires the result file. Task 1628 separately exercises exact
source/member kinds derived from the frozen hydration/lowering rules and a
non-copying template binding overlay. New inspection code does not change the
frozen source, generated inputs, expected pins or strict consumer admission.
Actual corrected results and complete category accounting remain required.

Both exact `underclass/gpt-6-luna` workers remain unavailable with the retained
quota errors; no model was substituted. No protected attempt has been launched.

## Complete current static contracts observed (2026-11-16)

Parent-owned task 1629 completed Success with
`STATIC_TEMPLATE_REVIEW_CLEAR` and an empty combined static blocker list.
Actual coverage is 125,355 selected recipe steps, 2,290 ControllerSpecs,
25,009 generated templates and 2,533,604 scoped static words. Every exact
producer/reference, constructor, word and source-role diagnostic category is
empty. All 816 distinct source references have consumer-derived kinds; all
11,153 source-reference occurrences are covered. The 6,167 runtime-generated
invocations have individually checked Produced Text requests and Produced
Executable consumers, but no actual runtime payload bytes were observed.

All 132 StageX lineage edges into pre-existing sources/patches were checked as
authenticated admission edges under the frozen consumer, not fictitious seed
production. All 341 native derivation-output edges are declared and consumed;
the Linux-header Make edge is traced through the selected output Tree's exact
catalog-authenticated `bin/make` Produced Executable to both selected header
Exec consumers. All 37 non-Exec native nodes are authenticated fetcher
materializations with the consumer-required output kinds. Raw mis-scoped word
flags and all 71 static operand-unused step diagnostics remain retained, not
silently deleted. Operand use is distinct from declared ordering/context input
validation; this result does not claim minimal authority.

`evidence/2026-11-15-leviathan-869a461d/current-inspection-summary.json` retains
the complete compact per-criterion counts, measured verdicts, command/result
links, source/runner/subject identities and current blocker. Complete earlier
blocked and final reports are retained as exact-byte `.json.gz` artifacts;
`graph-review/compressed-report-manifest.json` binds raw and compressed hashes
and verified decompression roundtrips. Failed inspector runs and their exact
source versions remain separate from successful measurements.

The immediate outcome is complete parent-owned bounded physical/static
qualification of the current inert cohort, not cold-proof completion. The
durable capability is explicit-subject, consumer-scoped reusable inspection;
the maintenance owner remains Mantle's source-built proof/operator workflow
(ADR 0155). Repeatability evidence is the retained four-argument commands,
exact inspector/tool/consumer hashes and actual tasks 1624, 1625 and 1629.

The required independent exact `underclass/gpt-6-luna` review remains blocked
by the retained `429 quota_exhausted` errors. No model was substituted and
parent results are not independent acceptance. Protected provider/stage1/stage2
execution, runtime decode/results, action-trust reconciliation, the verified
v2 receipt and native signed-row qualification remain unproven. I3–I5 and
proof-dependent V2/V3 remain open; no protected launch or lifecycle archive
is authorized by these static results.

## Authorized independent review continuation (2026-11-16)

The user explicitly authorized two independent `underclass/gpt-6-sol` reviews
of the same complete graph/template and physical/source criteria, replacing
only this cohort's earlier exact Luna model requirement. The direct answer is
retained in `evidence/2026-11-15-leviathan-869a461d/independent-review-model-authorization.json`.
No coverage, launch precondition, independent pin, resource bound or proof claim
was relaxed. Earlier Luna quota failures remain historical failures.

Detached remote task 1631 was recovered as Success without rerunning it.
Its complete captured transcript matches the remote bytes, and all four Cairn
reports are valid: validation plus proposal/design/tasks gates. The recovery
receipt is `inspection-publication-cairn-fixed-recovery-result.json` under the
same evidence root; it supersedes only the earlier unknown transport result.

`IndependentGraph869` and `IndependentPhysical869` are conducting the two
read-only current-subject reviews in separate evidence directories. Parent
checks and a model's text do not grant acceptance: all findings and uncovered
criteria must be integrated before a fresh cold attempt can be selected.

The current-cohort held driver reuses the sealed launch-review hash interface.
Its source/runner/profile/tool identities and 24-hour/200-GB bounds are explicit;
it uses a fresh proof/alias namespace and no development/cache/resume flags.
Shell syntax and transfer parity passed. An actual invalid review-hash argument
was rejected with exit 1 before the proof parent existed. This is a negative
launch-gate observation only; the proof is not queued or launched.

## Independent inspector finding (2026-11-16)

The physical/source slice is bounded accepted for criteria 7-8 and published
with parent-checked report, inspector and frozen-consumer bindings. The graph
slice rejected the final inspector's missing qualified Mantle-unit lookup and
omitted failure tally. Both actual binary matches were independently observed;
no product mismatch is asserted. The historical static-clear label is not a
launch prerequisite while this finding remains open.

The rejected inspector/review are retained. Inspector-only correction and
positive/negative in-memory checks are queued as detached inert task 1632,
followed by full unchanged-cohort graph/template reinspection and independent
re-review. No source/cohort/pin change or proof launch follows from this work.

## Corrected inspector and complete independent review observed (2026-11-16)

Task 1632 completed Success: two actual canonical-binary positives, eight
isolated negative rejections, and complete graph/template static-clear
measurement. Both binary/semantic matches are now enforced and true. All
14 selected-input and 30 frozen graph-consumer byte bindings were unchanged.
Full report transport and compressed-template roundtrip parity passed;
incomplete reader excerpts were rejected, not used as evidence.

Renewed exact-model graph acceptance covers complete criteria 1-6, with no
remaining static finding. Parent checked all 35 counter rows and 18 actual
artifact/consumer bindings plus subject, recipe, raw-template and transport
identities. Together with the separately accepted physical/source criteria
7-8, all required review criteria are integrated. The original graph rejection
is retained. Current Cairn gates and sealed review/fresh launch preconditions
remain before protected execution; I3-I5 and proof-dependent V2/V3 stay open.

## Native-row operator consumer prerequisites

- [ ] [serial] N1 Implement and verify receipt-bound cold StageX snapshot/projection and fresh signed adoption, then the complete five-row source-selected qualification and original receipt-validation workflow. r[bootstrap_inventory.cold_stagex_native_row_adoption]
- [ ] [serial] N2 Publish and freeze the changed source, qualify isolated runner and canonical targets, regenerate its source-only cohort and complete fresh current-data review before final changed-source proof or native-row promotion. r[bootstrap_inventory.cold_stagex_native_row_adoption]

The existing 869a461d attempt remains an immutable earlier-cohort run. Its
result cannot qualify changed Rust/operator source. No legacy task1580 retry,
old-output adoption, expected-pin refresh, partial-row acceptance or new v2
claim is authorized by these implementation prerequisites.

## Signed-adoption mechanisms and seed-contract repair observed (2026-11-16)

Cold task 1633 failed before protected provider execution. Its retained staging,
plan and absent successful aliases remain unchanged; the disposition is
`evidence/2026-11-15-leviathan-869a461d/cold-proof-1633-disposition.json`.
The retained inventory identifies 1013 invalid local seeds in each Mantle stage:
1012 native continuations and one generated Rust probe narrowed a Produced
tree's existing 2147483648-byte authority to a 536870912-byte output contract.
Both kinds were already Tree. The authors now retain the original bound;
generic exact-kind and bound admission remain unchanged.

Task 1640 executed five native regressions, one generated-probe regression and
one strict seed-admission regression successfully, plus formatting and strict
library/test Clippy. It subsequently failed on the independent script manifest's
missing vendored-Git lockfile, not those completed checks. The two operator
sources now use regular workspace binary targets and the existing lockfile and
patches. Task 1641 passed formatting, strict selected-binary Clippy, release
builds and the actual adoption, five-row checker and Python operator CLI surfaces.

The adoption and receipt-verification legs in task 1637 passed nine and eighteen
tests respectively before its separately retained Clippy failure. Three Python
regressions cover genuine diagnostic rejection versus launch/signal failure,
streamed escaped profile metadata and invalid source/manifest selection. The
complete-source guard smoke accepted an unchanged tree and rejected an imported
library mutation outside the row-file subset. These are mechanism observations,
not evidence that a genuine cold proof or native-row qualification completed.

Commands, transcripts, failed attempts and measured verdicts are retained in
`evidence/2026-11-16-signed-stagex-adoption/`. ADR 0156 and the operator workflow
own the repeatable snapshot, signed-adoption and five-row qualification path.
The user separately authorized two fresh `underclass/gpt-6-sol` reviews for the
replacement signed-adoption/seed-repair cohort; no old review is reused and no
coverage or launch gate is relaxed.

N1's real completed-proof adoption and all five native rows, N2's published
fresh qualification/regeneration/review, I3-I5 and proof-dependent V2/V3 remain
open. A warm-target developer check cannot qualify the changed frozen source.

## Replacement-source inventory gate observed (2026-11-16)

Published source `48afe43229ebccdb8d24ae382e8dabb59281dd71` and its exact Git
archive passed transfer parity; task 1642 assembled the complete source with
the unchanged independently pinned original AWS tree. Canonical task 1643
passed its selected release, bootstrap-parity, schema-mechanism, receipt,
adoption, Cargo-free, closure, self-build and release-core test legs, then the
actual machine-schema checker rejected the new adoption module's missing
producer-family inventory decision. Task 1645 was DependencyFailed and did
not execute preparation. This source remains an immutable unqualified
predecessor, not a current prepared or accepted proof cohort.

The operator's versioned report now has its own
`bootstrap.cold-stagex-adoption-report` compatibility classification and public
producer marker. It is not mislabeled as an internal-only encoding, nor added
to the curated generated-schema cohort. The existing Rust owner, receipt-bound
consumer, exact-version rejection and proof/store non-claims remain authoritative.
Nickel typechecking passed and the narrow current-root gate reported
`machine schema contract check: PASS (24 contracted, 66 classified)`; the exact
command/output is retained in
`evidence/2026-11-16-signed-stagex-adoption/producer-inventory-check.log`.

The changed source still requires a newly published freeze, absent runner and
canonical targets, successful regeneration and the full fresh independent
review before protected execution. No 48afe432 qualification or unexecuted
inspector port can be promoted to the final replacement subject.

## Declared operator source-capture repair observed (2026-11-16)

The d911681e runner and canonical qualifications passed, but preparation task
1649 rejected the operator target under uncaptured `scripts/`. Its partial
staging and dependent fact/transport rejections are retained. Both operator
sources now live under the already admitted `tools/` tree; manifest, documentation
and producer-family callers migrate together. Capture and target admission are
unchanged.

Task 1651 passed package formatting, strict selected-binary release Clippy,
release builds and the actual operator CLI surfaces. Its full-workspace native
plan exceeded the check-specific 300-second deadline, so that part is not a
successful planning receipt. Task 1652 separately exercised the exact native
target-admission core with real manifest/filesystem observations from an owned
copy of the retained capture: both moved targets are admitted and `scripts/`
remains absent. The current machine gate passed with 24 contracted and 67
classified families. Complete commands, reproduction source data, outputs and
transfer identities are in `evidence/2026-11-16-signed-stagex-adoption/`.

Maintenance owner: Mantle's source-capture and operator adapters. The durable
repair makes the declared binary sources available to the existing capture
contract rather than adding a second capture policy. A new published freeze,
isolated qualification, successful source-only preparation and full fresh
independent review still precede protected execution. N1-N2, I3-I5 and
proof-dependent V2/V3 remain open.

## Frozen captured-operator replacement qualified (2026-11-16)

Published product source `b520cf5791fb7638ab78ed59f42d890ccd75dc30` has exact
Git-archive transfer parity. Complete-source assembly task 1653 retained the
unchanged original AWS pins and produced normalized complete-source identity
`7d3a672bd38452322d43ff9465ddc8acd4d43c9776f152c3c6471a479bba4329`.

Distinct absent-target canonical task 1654 passed all ten selected test legs,
the current machine gate (24 contracted, 67 classified) and four valid Cairn
reports. Absent-target runner task 1655 passed formatting, strict selected
release Clippy, release builds, actual operator CLI surfaces and the captured
result `270 passed; 0 failed; 27 ignored`. Both qualifications retained the same
complete-source identity before and after execution. All three installed
regular binaries and their exact BLAKE3 identities are recorded in
`evidence/2026-11-16-leviathan-b520cf57/`.

These observations qualify the frozen code and outer runner mechanisms only.
Source-program preparation, current-data inspection, two fresh authorized-model
reviews, protected stages, v2 receipt and all five native rows remain separate
acceptance obligations. No predecessor acceptance or failed staging is reused;
N1-N2, I3-I5 and proof-dependent V2/V3 remain open.

## Source-owned selector facts and corrected inspection observed (2026-11-16)

The original b520cf57 graph review rejected missing criterion-6 measurements;
the original physical review rejected incomplete original-record body coverage.
Those rejections and failed checker versions remain retained. Corrected physical
inspection compares all 177 original native bodies across the preauthor plan,
augmented closure and launch-input profile; the separate corrected preauthor
report covers all three directory roles, 70,794 embedded leaves and four file
roles. These remain bounded observations of the unchanged predecessor cohort.

ADR 0157 and the operator proof guide now require authenticated source-only
unit plans retained before Mantle recipe lowering. Developer task 1664 passed
package formatting, three focused tests, strict release Clippy and a release
build; preparation 1667 emitted both real 1,047-unit plans. Task 1679 authenticated
both complete 8,740,623-byte plan members against the actual profile and passed
all two package exclusions, 2,386 Cargo feature-environment contexts and 2,098
compiler cfg comparisons with zero invalid/uncovered entries. Each stage maps
1,049 authored compilers to all 1,047 source derivations, with two explicitly
recorded unique-metadata associations and no orphan derivations. Nine actual
in-memory recipe mutations were rejected, including removed continuation and
invocation-template feature flags and a missing semantic build-script root.

Separate predecessor helper tasks 1675/1674 passed all 1,098 owning-stage
authorities and six rejection cases. Complete reports, source versions, failed
attempts, exact commands and matching transport identities are retained in
`evidence/2026-11-16-unit-plan-facts/`. The source-only unit-plan metadata stays
inside the existing private proof-artifact family; its machine gate passed
with 24 contracted and 67 classified families.

Maintenance owner: Mantle's source-program authoring and proof-inspection
adapters. The durable capability is a repeatable, source-bound selector
comparison without a second feature resolver. These developer and predecessor
observations do not qualify changed frozen source or grant launch authority.
A new published freeze, distinct absent runner/canonical targets, genuine
source-only regeneration, complete current-data inspection and both fresh
authorized-model reviews still precede protected execution. N1-N2, I3-I5 and
proof-dependent V2/V3 remain open.

## Source-unit-plan replacement frozen qualifications observed (2026-11-16)

Published code `df51377d592bf6a3412b436a10e8fc708e8dbe3d` retains ADR 0157's
upstream unit plans. Its exact Git archive passed transport; complete-source
assembly 1680 preserves the independently pinned original AWS source and has
normalized identity `3ee5b549a06a9ba23f5e8f67fe83e8e7c8ae872d0f601cdb9800e4864b6e8bba`.

Distinct absent-target canonical 1681 passed all ten selected test legs, the
24-contracted/67-classified machine gate and all four valid Cairn reports.
Distinct absent-target runner 1682 passed package formatting, strict selected
release Clippy, three release builds, actual operator CLI surfaces and the
captured result `270 passed; 0 failed; 27 ignored`. Both retained the exact
complete source identity before/after qualification. Installed Mantle identity
is `17358d92b9ce16a00fc4407334bb8c7745ff127cd2f515ec4fa74fcf6a3b24a6`.
Complete transcripts, binary identities and matching output transports are in
`evidence/2026-11-16-leviathan-df51377d/`.

Dependent source-only preparation 1683 and subject-fact capture 1684 are separate
unfinished prerequisites. The complete corrected graph/helper/selector and
physical inspector bundle is transferred, not executed or accepted on absent
current prepared data. Both fresh authorized-model reviews still precede any
protected launch. No provider, stage equality, v2 receipt, native-row acceptance
or release claim follows; N1-N2, I3-I5 and proof-dependent V2/V3 remain open.

