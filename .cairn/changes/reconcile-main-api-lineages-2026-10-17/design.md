# Design: Current API-aware main reconciliation

## Status and profile

Current integration is blocked on executable/generator/gate evidence, not
accepted or archived. These artifacts were manually authored using the actual
`cairn-policy/default.ncl` spec-driven v1 template obligations. Metadata omits
`workflow_profile`: Cairn's compatibility selection uses the repository default
`spec-driven`. It does not falsely record immutable `source: change_create`.
No Cairn command-creation receipt or gate result is asserted.

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
truth. Fresh actual generator output and mode-checks are required before
installing generated JSON/docs. The completed pinned Nickel inventory export
alone does not resolve the two remaining generated marker files or establish
root/generator build success. No catalog identity is hand-spliced.

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

Direct current-cohort native validation/proposal/design checks returned
`valid: true` for the authored documentary snapshot; exact cohort binding and
outputs are in `evidence/native-document-validation.json`. No tasks/probes ran.

Fresh root plus two-generator build, fresh CLI/contract/matrix smoke, ordinary
GCC4.0 bridge → scoped fmt → vendor-excluded strict Clippy → workspace lib/tests,
extra affected-package fmt, final-source Cairn/G3 including tasks, and
Nix/TigerStyle gates are PENDING. They require their reviewed bounded execution
route; documentary checks neither admit product execution nor satisfy G3.

## Risks and claim boundary

Caller migration can compile in a small harness while a root consumer remains
broken. Formatting can change the subject after a smoke. Installed JAR behavior
can differ from pinned reviewed tools. Store publication is weaker than cache
trust admission. These risks require the pending checks; none is discharged by
metadata, a merged source tree, selected test output, or this ADR.
