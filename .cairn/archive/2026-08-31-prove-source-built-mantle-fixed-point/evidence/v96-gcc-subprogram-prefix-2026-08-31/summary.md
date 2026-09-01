# V96 action-scope reconciliation failure

## Verdict

V96 passed the canonical GCC subprogram boundary and completed the native
combined-topology executor's selected stage1 units. Protected execution observed
5,190 events across 836 actions with no authority violations.

Reconciliation failed because the action plan also required 26 compile actions
for review-graph derivations that this execution mode did not schedule.

This attempt does not prove accepted stage1, stage2, fixed-point equality, the
final receipt, or complete trust.

## Bound inputs

- Source commit: `29bfadcab89321b07070cc9df5a898a97fe2e5fb`
- Orchestrator BLAKE3:
  `c3caa14734ba909119e9269ac2e2bb31fbe3d2a1a97fbebb23203250cb3e3706`
- Ready source-profile BLAKE3:
  `bfeaad9a4774e715561f7299a907c5255bbb813a9e940fb40edc415e27e7b2ed`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 705,673,805,824

The source and binary transfers had exact checksum and round-trip parity. The
profile verified `Ready` with zero missing, stale, unsupported, or untrusted
records.

## Passed boundaries

V96 restored the immutable provider checkpoint, relocated all 17 closure
members, and validated the binding-owned rustc runtime.

The canonical GCC `-B` prefix removed the parent-component `cc1` request. That
V95 failure did not recur.

The topology executor completed its selected units. The protected audit and
reconciliation record:

- planned actions: 862;
- matched actions: 836;
- observed events: 5,190;
- matched events: 5,190;
- unknown events: 0;
- denied events: 0;
- drifted events: 0;
- overbound actions: 0;
- missing actions: 26.

`stage1-rust-action-audit.json` preserves every raw and assigned event.

## Root cause

`SourceBuiltRustActionRuntime` adapted every derivation in the native review
graph. `execute_rust_unit_topology_inner` executed only the deterministic
combined-topology scope: supported target, host, and host-dependency units.

The wider review graph contained 26 unscheduled derivations. Their required
compile actions could not produce kernel events, so reconciliation correctly
failed.

`missing-action-summary.tsv` maps each missing action to its package and target.
The set includes unrelated workspace targets and their otherwise unscheduled
registry closure. `summarize-missing-rust-actions.rs` is the checked analysis
script.

## Decision

ADR 0099 binds action planning to the same combined-topology execution scope.

Before policy installation, Mantle now derives the exact selected unit IDs,
selects only those derivations, and builds producer indexes only from that set.
Empty, unknown, duplicate, or incomplete selections fail.

Every planned action still requires a positive event count. Mantle does not
synthesize observations or mark missing actions optional.

## Validation

`post-repair-validation.log` records Rust 2024 formatting and these passing
suites, each with one test thread where supervision is involved:

- 9 combined-topology tests;
- 14 Rust action-plan tests;
- 2 Rust action-shell tests;
- 67 cargo-free self-build tests.

Positive coverage excludes an unsupported derivation from the action scope.
Negative coverage rejects empty and unknown scopes. The ptrace runtime test
still produces complete reconciliation.

## Cleanup

- `cleanup-v95-before-v96.txt` records no-follow removal of the committed V95
  staging root.
- `cleanup-v95-profile-before-v96.txt` records removal of the superseded V95
  profile after V96 verified `Ready`.

Neither cleanup changed regular-file modes.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. Build and transfer a
new release binary, refresh a Ready profile, and run a fresh promoted proof.
Preserve V96 until the next proof no longer needs its complete stage1 audit.
