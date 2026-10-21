# Design: Current API-aware main reconciliation

## Status and profile

Current V2/V3/V4 observations have completed on their separately recorded
post-cleanup subjects; native G3 passed on declared documentary input943cae0a…
before these additive outcome records. G1/G2/P1 and end-to-end acceptance remain
pending. A later tiny G2 scratch witness passed, but latest full G1 admission
refused before enqueue; full G1/G2 remain unexecuted despite static fmt PASS.
These artifacts were manually authored using the actual
`cairn-policy/default.ncl` spec-driven v1 template obligations. Metadata omits
`workflow_profile`: the repository default remains `spec-driven`; no immutable
`source: change_create` receipt is invented.

## Source and component ownership

Both named parents in `proposal.md` remain the integration inputs. Functional
core decisions stay in no-std/alloc core crates; filesystem/process/network,
trust-policy reads, execution and publication stay in the existing std shell.
There is no second DTO convention or blanket side selection.

### Cargo API cutover

`mantle-cargo-unit-plan::UnitBinding.build_script_run` owns the optional
`BuildScriptRunBinding { links, native_inputs, native_env, native_path }`.
`PlanBindings.runtime_inputs` preserves explicit ELF loader/library roots.
The Rust-plan extracted build-script module owns shared directive/native-input
vocabulary; the producer resolves admitted facts and the helper executes the
bounded child. Native inputs do not authorize ambient host discovery. Cargo
still owns graph/feature partitions; ADR 0113 constrains equivalent host-unit
selection. This is not general Cargo compatibility or replay of old app42.

### Android contract and timestamp boundaries

The published root APK plan and `crunch_android_core::reviewed` represent
different reviewed source-authority contracts, not aliases for one DTO.
Nickel selects the appropriate authoring surface; adapters keep those authority
checks and signing declarations reachable. Store password and key-password
files remain explicit inputs. ADRs 0110–0113 are retained unchanged here.

Both routes require locale C, UTC and integral entry epochs starting at
315532802 (1980-01-01 00:00:02). Published JAR creation uses staged member
mtimes and admits through 4354819199 (2107-12-31 23:59:59). Reviewed JAR update
uses `--date` and admits only through 4102444799 (2099-12-31 23:59:59).
ZIP DOS timestamps have two-second precision: nominal final second 59 is
observed as 58, not evidence of exact one-second preservation. Neither widen
the reviewed route to 2107 nor shrink the published route to 2099 to make
DTOs appear interchangeable. Installed OpenJDK observations are not pinned
reviewed Temurin or SDK execution evidence.

### Selected-prefix layered store authority

Signature fingerprints use the configured logical store prefix for roots and
references. Public layered listing and direct layered lookup apply the same
trust authority. Listing visits the writable overlay then bases in declared
order, preserves nearest-layer shadowing, and fails on invalid visible trust
rather than treating an empty low-level overlay stream as the composed list.
`StoreHandle::list_pathinfos_with_layer` owns that composed inspection path.
Casita policy is destination-owned; an already-open handle must not authorize
revoked signers from a stale cache, and invalid policy must not authorize GC.
Local action-result publication/discovery is not trusted remote reuse.

### Generated operator authority

Current Clap declarations plus `config/operator-surfaces.ncl` remain source of
truth. Actual fresh generator output was captured before exact catalog/reference
installation; actual root mode-check and the unmodified contract script then
passed184 commands. Raw descriptors, reviewed Nickel export and workflow were
already byte-identical and not rewritten. Portable profile comparison passed.
The starter remains review-only; no catalog identity was hand-spliced. Exact
subjects/output limits are in `evidence/post-cleanup-milestones-v1.md`.

### Evidence and resource epochs

This change owns a new current subject; active and archived older subjects
keep their own tasks, acceptance and raw receipts. Selected current receipts
are copied byte-for-byte, with original commands/results retained. They predate
final scoped formatting and do not prove a later executable subject.

The original pool-availability baseline (F0) and caps remain frozen. Proven
exclusive-growth credit (E) is zero; the uncaptured frees/concurrent-allocation
correction (H) is unknown. Unknown H does not impose a new hard-quota
requirement: the original legacy guard with zero credit already exceeds its
threshold. A current owned-process/lock observation is point evidence, not proof
that old compiler descendants or opaque Nix daemon work are absent. The parent
reports read-only review t19 exhausted the pre-F0 ZFS snapshot/bookmark route:
both
datasets returned no retained markers. Current absolute `written` totals are
not original-epoch deltas; no observable original epoch is invented. Exact
parent-reported observations and unchanged F0/prebaseline values are recorded
in `evidence/current-observations.md`, not fabricated raw command output.
Accounting preparation neither admits product execution nor retroactively
turns a resource stop into a compiler pass.

The fresh unchanged3150 global two-root incremental inventory found zero
eligible candidates and retained238 protected/working entries; there was no
second apply. The completed original436 operation under F53A is CLOSED. A
separate explicit exact-artifact review then disposed only the brand-new,
completed one-off composed-list consumer created AFTER that operation, under
the existing preserved Cargo EXLOCK. All eight metadata identities passed
before six file unlinks and two empty owned-directory removals; no errors.
Its source/manifest/copied-lock snapshots and stdout remain BLAKE3-retained;
other R resources and old receipts/manifests remain untouched. Intentional
disposable-output absence requires reconstruction at fresh admitted paths,
not replay of the old exclusive-mkdir script.

Immediate cleanup before/after free40859074560B/shared83833913344B was unchanged;
target changed9866972672→9331496960B, temp270222848→270142464B, metadata2909696B
unchanged. ZERO physical free gain was observed; the shared stop83751862272B
remained exceeded by82051072B. Deduplicated gross267740160B and name-accounted
target shrink535475712B (two hardlink names) are not physical recovery. This is
ordinary owned-scaffold disposal, not another cache eviction, funding, guard
clearance or admission. Original F0/E0/caps/epoch remain frozen. Exact receipts
and scope are in `evidence/post-cleanup-milestones-v1.md`; no delayed resample
or automatic follow-on is authorized.

## Rejected alternatives

- Blanket ours/theirs: loses behavior and prerequisites of one parent.
- Parallel legacy/current aliases for Cargo: makes consumer migration ambiguous.
- One Android timestamp ceiling by DTO convenience: contradicts actual renderer
  operations and their route-specific boundary observations.
- Listing only the raw overlay or trusting only direct gets: hides base records
  or admits a signature under inconsistent prefix authority.
- Hand-merging generated IDs or borrowing old successful receipts: creates a
  result not produced by the current source/commands.
- New allocation baseline or caps to erase prior usage: violates frozen resource
  accounting and does not establish prior attribution.

## Verification and failure behavior

Selected current evidence records helper-native linking/execution and a missing
native negative; network/host-path namespace denial; published/reviewed Nickel
boundary admission/lowering; a real installed JAR upper-bound comparison;
selected-prefix layered get/list/reference success and wrong-prefix rejection;
and Casita signed persistence, revocation, action-result publication and stale
GC-plan rejection. Each receipt's scope and limitations remain explicit.

Older current-cohort validate/proposal/design observations in
`evidence/native-document-validation.json` remain historical; no tasks/probes
ran in that older BF95 observation. Actual four-command G3 is recorded below.

Actual post-cleanup root plus two-generator build, generator capture/exact
installation, root CLI/contract/matrix smoke and current helper/Android/store/
Casita observations passed on their declared subjects, before this documentary
update. The dedicated public consumer proved composed-list near/far shadow
selection rather than inferring it from lookup/cache tests. The existing helper
namespace fixture now honors TMPDIR and its actual body passed without skip.
See `evidence/post-cleanup-milestones-v1.md` and its separate exact receipt index.

Actual direct pinned native strict validate and proposal/design/tasks all
exited0 with valid:true/issues:[] (gates PASS), with declared17 documentary/policy
inputs unchanged across execution, binding
`943cae0a74f58b0b3a526acbe9feec1b6d665f1927765b289157ae4c980c7dac`.
This is the input BEFORE these additive outcome records, not a post-edit/full
production-tree identity. Full native streams and losslessly compacted COMPLETE
capture metadata are retained with explicit original/reconstructed wire hashes.
Tasks ordering was valid (seven Done/four Todo at capture); no probe declaration
or execution. G3's after:V4 ordering is independent of held G1/G2, so read-only
document/saved-evidence checks required no product/Nix/build admission. The raw
P1 blockers are not retconned: after recording G3, G1/G2/semantic acceptance
remain pending. Verification of these new documentary inputs is separate.

The later separately admitted tiny G2 scratch witness actually proved builder
`/build` mapping beneath original R/t through matching device0:47 and
zfs/datapool/mantle-store kernel mount rows. Recovery task6/controller/inner/
monitor child succeeded with no stop; final free41128689664B/shared83564298240B
left187564032B to frozen stop. This proves that tiny route, not full G2, fit,
cleanup-causal recovery, successor authority or daemon settlement. Source
`124ca12d1f1cc1a3a8be252f6d0f12f7f1ca2014d85700b3a8a545633f60bfe3`
and documentary `3cbaaa73105b8fca8e037fb8bbdc23b8a320c769dc2d64c9fcac89102b45fcb2`
identify witness/latest G1 subjects; retained terminal bindings are not a full
source-row snapshot. Earlier failed admission and ZERO immediate cleanup gain
remain true historical points, not contradictions or funding.

Latest full G1 admission refused exit75 BEFORE enqueue/bridge/Clippy/tests:
free40903507968B/shared83789479936B,37617664B above unchanged83751862272B stop;
target9331496960B/temp270171648B/metadata2994176B. No compiler failure is inferred.
Independent existing-cohort static checks passed: canonical17 v3 clean exit0,
extra12 and edition2024 gateway leaf once in v1;611 declared source/lock/config
inputs unchanged, binding
`4df6dbccf81c2c49b7b19b82450e674f720c231b3833e7707af2c74763f4ee91`.
Full v1/correction/v2/v3 receipts and ten raw streams are retained separately
in `evidence/final-verification-v1/index.json`. v1 canonical dispatch failed
before Cargo; cause unknown, not a proven login/BASH_ENV bug. v2 helper-init
diagnostics invalidate clean PASS despite exit0; v3's explicit child environment
resolved dispatch without source/profile edits. Passed extra12/gateway were not
rerun. Static fmt is not GCC4 bridge, strict Clippy, workspace tests or Nix gates.

Full G1, full G2 current-system flake check (already includes TigerStyle), and
P1 semantic acceptance/publication remain PENDING. G3 and separately authorized
static checks do not admit held product/Nix/build actions. Final native checks
against these newly frozen documentary inputs require separate parent release.

## Risks and claim boundary

Caller migration can compile in a small harness while a root consumer remains
broken. Formatting can change the subject after a smoke. Installed JAR behavior
can differ from pinned reviewed tools. Store publication is weaker than cache
trust admission. These risks require the pending checks; none is discharged by
metadata, a merged source tree, selected test output, or this ADR.

## Published SDK dependency reconciliation

The later integrated source is a distinct subject from the frozen historical
observations above. Pueue 1391 stopped at locked offline metadata before its
compiler checks because the new Valence dependency had no lock/source-map entry.
Pueue 1393 and 1396 then exposed the published Basalt/UCAN graph's exact zeroize
and BLAKE3 requirements. These failures do not invalidate the earlier bounded
receipts and do not establish current-source acceptance.

[ADR 0122](../../../adr/0122-use-native-blake3-streaming-at-snix-boundary.md)
records the local Snix streaming adaptation and common BLAKE3 pin cutover.
The SHA/Nix digest boundary, content identities, offline source admission and
owner authority checks remain unchanged. Cargo/Nix must regenerate dependency
inputs; focused stream and real store/build execution plus ordinary gates must
bind the final source before any release-baseline or lifecycle completion claim.


### Published Artifact source cohort

Artifact source declarations now select the published SSH source
`ssh://git@github.com/OnixResearch/onix-artifact.git` at
`5f3962d7471d1547e044f57daeda72f56621fd52`. The current source-admission
contract requires the four-package AGPL-3.0-or-later workspace, but Mantle's
resolved consumer set remains only `artifact-auth-core` and
`artifact-auth-ed25519`. Root, action-result, build, and release manifests must
agree; binding/transfer resolution, path substitutions, floating references,
and predecessor sources remain inadmissible. No second source or trust grant
is introduced.

The integration owner reports an independently fresh primary GitHub SSH clone
at that revision, with all 180 tracked published source files measured by
BLAKE3. Its Nix metadata is retained in Artifact at
`evidence/published-source-v1/primary-nix-metadata.json`; the measured NAR is
`sha256-Xr7ocFkPN15XYb6NA4bv3I9Q42NvoSYtXWfsrj7I3WY=`
(base16 `5ebee870590f375e5761be8d0386efdc8f50e3636fa1262d5d67ecae3ec8dd66`).
Mantle's current source assertion, lock check, and two reviewed recursive-hash
table rows bind that NAR. Core and Ed25519 `src/lib.rs` BLAKE3 identities remain
`839ea3e35429e2a1252020dd779b46ebd185b6911674036820da118d487d1d21`
and `7fd3987fe3ddc6d30efca37878952135f19905a00b52f349db237054be6b3633`.
Primary acquisition is not a claim of transferred-source/full-source gate
success, current Mantle consumer execution, or archive acceptance.

This is a declaration and admission-contract update, not a current Mantle
execution or acceptance receipt. The integration owner must regenerate locks
and vendor outputs with their owning tools, then exercise current consumers
and ordinary gates. The original accepted
2026-08-06 migration receipt, validator, rollback records, and fixture baselines
retain their original identities and claims. Their historical checks do not
establish acceptance of this new source cohort.

