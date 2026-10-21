# Current API-aware main reconciliation (2026-10-17)

Status: the reconciled source is published on `main`; the ordinary gate is
blocked by a pre-existing Nickel standard-library defect (issue 24), so no
acceptance, sync, archive, or release claim follows. This entry describes the
new current integration subject, not a release or acceptance of older proof
campaigns.

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

## Observed post-cleanup milestones

The additive [milestone record](../../.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/post-cleanup-milestones-v1.md)
and [dedicated receipt index](../../.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/post-cleanup-receipt-index-v1.json)
retain80 post-cleanup source/copy pairs:985966B raw +40305B index; raw/index/note
remain under unchanged1MiB combined. This append adds only witness11/19595B
and actual G1 admission/refusal2/3740B; old67 pairs/records and original29/
historical indexes remain unchanged and not reaudited. Complete static producer
metadata plus all ten streams use the separately authorized
[final-verification index](../../.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/final-verification-v1/index.json):
14 exact pairs/977629B raw +8171B index =985800B under fixed combined2MiB cap,
also reserved for separately released final native observations. Nothing from
the old post-cleanup payload was moved; native G3 lossless metadata is unchanged.

- **V2:** actual current root plus both generators built successfully, child
  exit0/Pueue Success/no resource stop, compiler finish7m52s, on source
  `e7edc03304aa9b35813a5d90495fb42a362f624ad4cd2daa8016405f960371a8`.
- **V3:** actual first-generator self-test and starter output, duplicate-checked
  portable-profile equality and reviewed Nickel1.17 export succeeded. Only
  exact captured catalog/reference bytes were installed, with generator-owned
  identity `12e74923e80c41b28cede8a16c566e129a9ebce7b1fa3bbffe2a908b876fb7c7`.
  Descriptors, embedded inventory JSON and workflow were already byte-identical
  and untouched. Actual root mode-check/unmodified operator script PASS184 and
  limited matrix checks passed on source
  `8d73114ffd5669121a3a727ca1157b704b0dc6f1f4c56a88221b424f19006209`;
  referenced historical fixtures were not replayed.
- **V4:** fresh helper native42/missing-native/quoted-cfg and actual namespace
  body passed; the existing test now honors invocation TMPDIR via fixture_root
  instead of writing a crate-local target fixture. Namespace result1passed,
  0failed,8filtered, no skip; no new test or production API was added.
  Both Android authoring routes' seven timestamp cases and authority/signing
  declaration negatives, installed JAR route bounds, selected-prefix
  get/list/reference and actual composed-list duplicate shadows, and Casita
  persistence/revocation/policy-GC/stale-plan/local-publication checks passed.
  Three focused store tests each passed1/0 with453filtered. Both actual V4 legs
  exited0/no stop on source
  `91e664eca5ccbab46b3bfe36d0c826fbc880f346c471d183db7654cc1c6860cc`,
  before this documentary update.

The composed-list consumer selected the full good nearer PathInfo/node with
Overlay layer and Base1 Conflicting shadow despite a wrong-prefix farther
duplicate; a wrong-prefix nearer record rejected despite a good farther one.
Layer precedence is selection authority, not output vector sorting. Casita
local Published→Duplicate/discovery is not trusted remote reuse. Installed
OpenJDK observations are not pinned reviewed Temurin/SDK/device/APK/signing
execution. These bounded observations do not prove general Cargo compatibility,
compiler correctness, global reproducibility, complete Mantle/bootstrap or
release eligibility. Existing component owners and canonical commands remain
responsible; repeatability requires fresh paths/current bindings/resource
admission, not replay of fixed-path historical wrappers.

## Actual bounded native G3

Direct pinned existing Cairn strict validate and proposal/design/tasks all
exited0/valid:true/issues:[] (gates PASS); declared17 documentary/policy inputs
were unchanged across execution, binding
`943cae0a74f58b0b3a526acbe9feec1b6d665f1927765b289157ae4c980c7dac`.
This input precedes these additive outcome records, not a post-edit or full
production-tree identity. Tasks ordering was valid, seven Done/four Todo at
capture; its raw P1 blockers G1/G2/G3 remain historical input facts. After
recording G3, P1 still awaits G1/G2 and semantic acceptance. Probe declarations
were absent, probes empty, none executed. The separately released read-only
document/saved-evidence path was independent of held product/Nix/builds; it
performed no product launch, queue/R write, admission resample or daemon proof.
The older BF95 three-command observation is not this actual four-command G3.

## Explicitly pending and current resource blocker

Full G1 GCC4 bridge/strict Clippy/serialized workspace lib-tests, full G2
Nix/TigerStyle, and P1 semantic acceptance/sole-owner publication remain pending.
Independent static fmt components passed: canonical17 v3 clean exit0/stdout0/
stderr356B only expected announcement; extra12 and edition2024 gateway leaf
each exit0/empty streams once in v1, not rerun. All611 declared inputs remained
unchanged, binding
`4df6dbccf81c2c49b7b19b82450e674f720c231b3833e7707af2c74763f4ee91`.
v1 canonical dispatch failed before Cargo, cause UNKNOWN (not a proven login/
BASH_ENV bug); v2 exit0 included helper-init diagnostics and is not clean PASS.
v3 changed only explicit child dispatch environment, not source/profile.
Full receipts retain all611 rows, timings/argv/environment and raw streams.

The old tiny-witness admission exit75/no launch remains historical. A later
separately admitted once-only tiny witness was PROVEN: fresh local builder
`/build` matched host dev0:47/zfs/datapool/mantle-store and mapped component-wise
beneath original R/t; task6/controller/inner/monitor child0, stop:null. Final
free41128689664B/shared83564298240B left187564032B to unchanged shared stop.
This is actual physical scratch mapping, not full G2 PASS or full-build fit.
Full current-system flake check already includes pinned TigerStyle; neither
that full rail nor G1 was replaced by the tiny witness or static formatting.

Latest G1 admission refused exit75 BEFORE enqueue/bridge/Clippy/tests:
free40903507968B/shared83789479936B,37617664B above unchanged83751862272B stop;
target9331496960B/temp270171648B/metadata2994176B. No compiler failure, repeat
admission or space poll is inferred. Witness/latest G1 source binding
`124ca12d1f1cc1a3a8be252f6d0f12f7f1ca2014d85700b3a8a545633f60bfe3`
and documentary binding `3cbaaa73105b8fca8e037fb8bbdc23b8a320c769dc2d64c9fcac89102b45fcb2`
are terminal identifiers, not retained full source rows or atomic snapshot.

Zero-eligible/238-retained inventory/no second apply and CLOSED original436/
F53A operation remain unchanged. Separate ordinary disposal of only the new
completed consumer left immediate physical free gain ZERO;267740160B inode
gross/535475712B target-name shrink are not recovered capacity or causal funding.
Retained source/manifest/copied-lock/stdout enable reconstruction at fresh
admitted paths, not replay of exclusive-output scripts. F0/E0/caps/epoch/pins/
profiles stay frozen; no autochain, hard-quota, successor or daemon-settlement
claim follows. HEAD034a2dbb preserves both parents, not publication acceptance.

G3 is recorded only on its observed input; separate verification against these
newly updated documentary inputs is still pending. No accepted-spec sync,
archive, overall semantic/lifecycle completion or release is asserted.
ADR0114 remains Proposed; historical pending campaigns retain their subjects.

## Published baseline and the remaining gate blocker

The reconciled source is published. The sole Git owner recorded the ordinary
merge and the normal main push with both histories; the exact push and
`ls-remote` output is retained in
`.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/publication-v1/push.txt`.

On the admitted source snapshot the guarded verification recorded a green
compile admission over all targets, the canonical first-party rustfmt, strict
Clippy with the vendored members excluded, and the focused regressions for the
artifact-auth, source-program, controller-authority, recipe-authority, and
protected-executor paths. The local checkout was byte-verified against that
snapshot before each commit.

The ordinary first-party gate itself was blocked by a pre-existing defect, not
by this reconciliation. The pinned Nickel 1.17.0 did not evaluate the
standard library: a record contract may carry at most one field without a
definition, and Nickel records are recursive, so a field whose value is the
same identifier as the field name is an infinite recursion. The published
baseline `43ccd6884` fails the same way, so every Nickel-evaluating
workspace test failed. The analysis was recorded in
`docs/nickel-stdlib-contract-defect.md` and issue 24.

That defect is now repaired and published. Every published plan shape is
restated without changing what it declares: a shape whose fields are all
required became a record type, and a shape with defaults or optional fields
became `std.contract.Sequence` of an open defaults contract and the new
`contracts.Shape` whole-shape check, which requires the declared fields,
rejects undeclared ones, and type-checks the required ones. Module export
records alias the names they re-export. The decision is ADR 0124 and the
repair is tracked by the Cairn change `repair-nickel-contract-shapes`. The
Rust glue layer remains the admission authority for every evaluated plan.

The gate result for the repaired revision is recorded with its transcript in
`.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/final-gate-v1/`.
That transcript is evidence for the recorded revision only. It is not a
bootstrap, reproducibility, or parity claim, and the reconciliation's G1 and
G2 acceptance still require their own criteria.
