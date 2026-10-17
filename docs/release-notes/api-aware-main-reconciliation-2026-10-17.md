# Current API-aware main reconciliation (2026-10-17)

Status: source integration in progress; end-to-end acceptance and publication
are not established. This entry describes the new current integration subject,
not a release or acceptance of older proof campaigns.

## Current contract decisions

- Cargo build-script consumers use one `build_script_run` binding with explicit
  native inputs/environment/PATH and separate declared tool runtime roots.
  Shared Rust-plan directives remain extracted; no duplicate legacy run DTO.
- Published and reviewed Android authoring remain explicit routes with their
  respective source authority. Their minimum entry epoch is 315532802;
  published staged-mtime JAR creation ends at 4354819199, while reviewed JAR
  `--date` update ends at 4102444799. ZIP seconds have two-second precision.
- Layered PathInfo get/list/reference trust uses the configured logical prefix;
  composed listing includes admitted base records in layer precedence and
  rejects visible wrong-prefix signatures. Casita keeps destination-owned trust.
- Operator catalog identities remain generator-owned by current Clap and
  reviewed Nickel, not manually combined from the two histories.

See [ADR 0114](../../adr/0114-reconcile-lineage-apis-with-explicit-contract-boundaries.md)
and the [current integration change](../../.cairn/changes/reconcile-main-api-lineages-2026-10-17/proposal.md).

## Selected observed current behavior

The [current evidence index](../../.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/current-observations.md)
retains current helper execution yielding `current-native-answer=42`, native
namespace denial, both embedded Nickel authoring routes' positive/negative
boundaries, installed JAR upper-bound/ZIP observations, selected-prefix
get/list/reference success and wrong-prefix rejection, and Casita signed
persistence/revocation/action-result publication/stale-GC-plan checks.

These are API/helper/tool observations on their recorded pre-final-format
subjects, not fresh root CLI execution, complete APK/SDK/signing execution,
trusted remote cache reuse, pinned reviewed JDK proof, general Cargo parity,
compiler correctness, global reproducibility or release eligibility.

Direct native Cairn validation/proposal/design checks, bound to the repository's
required existing cohort, each exited 0 with `valid: true` for a documentary
snapshot. Exact commands/cohort/output identities are in the current evidence
index's linked native documentary receipt. No tasks/probe/sync/archive command
ran; these checks do not satisfy final-source G3 or product acceptance.

## Explicitly pending

The actual root plus two-generator attempt was resource-stopped, not successful.
Fresh root/generator build, current CLI/generated capture and installs,
operator mode-check/contract/matrix smoke, ordinary quality and extra affected
format checks, final-source Cairn/tasks and Nix/TigerStyle gates remain PENDING.
Product execution is held during resource-accounting preparation/review.
The original pool-availability baseline and caps are unchanged. Proven
exclusive-growth credit is zero; the correction for uncaptured frees/concurrent
allocation is unknown. That unknown correction does not impose a new hard-quota
requirement: the original legacy guard with zero credit already exceeds its
threshold. The current evidence index records exact values and provenance.
Process point observations do not establish old compiler-descendant/daemon
emptiness. No accepted spec sync, archive, main push, bootstrap or release
success is asserted. Historical raw receipts and older task statuses are
unchanged by this entry.

## Current recovery outcome

The parent accepted V1's point static API review; its original source binding
does not cover subsequent documentary edits. After the prior runtime owner's
session died, the existing recovery session received sole current custody.
Fresh isolated Pueue task0 ended with actual `Failed:75`: unchanged OFFLINE
shared-pool guard (AE78 result) refused before any Nix child/compiler/product launch. Original
F0 shared growth86195830784B exceeded the78GiB stop/80GiB cap; free was
38497157120B. Invocation-local `eval-cache=false` was verified by a value-only
query, not by successful `nix develop`. Old task5 exit remains UNKNOWN.

The [recovery evidence](../../.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/recovery-and-guard-refusal-2026-10-17.md)
retains exact raw identities, resource gap and custody. V2 and every later
product/gate/publication task remain pending; generated catalog/reference
identity conflicts remain unresolved. No retry, cleanup, new allocation or
cap/baseline reset is authorized. Fresh native documentary-only outcomes are
linked in `evidence/recovery-document-validation.json` within that change;
they do not confer G3, lifecycle or release acceptance.
