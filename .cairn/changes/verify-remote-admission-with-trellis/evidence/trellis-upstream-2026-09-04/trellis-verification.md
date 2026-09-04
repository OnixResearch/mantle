# Fenced Attempt Admission Verification

## Result

The finite fenced-attempt implementation is complete. Strict validation and
the proposal, design, and tasks gates pass. Sync created the accepted
specification, and archive execution completed under
`2026-09-03-add-fenced-attempt-admission-primitives`.

The public `fenced_attempt_admission` facade composes narrow model, relation,
transition, decision, proof, and test components. Every production file has
fewer than 300 lines. The functional core performs no filesystem, environment,
clock, network, process, persistence, serialization, hashing, or printing
operation.

The model provides:

- seven closed phases and six closed report kinds;
- epoch classification into current, stale, and future stances;
- job and run identity rejection before fence arithmetic;
- event classification into new, replayed, and conflict outcomes;
- terminal closure across completed, failed, and superseded phases;
- worker scope gates for state-changing reports;
- completion gates requiring result-ready phase, worker scope, output scope,
  and exact retained result identity;
- bounded event history of at most 4,096 entries;
- next-epoch arithmetic that rejects at the fixed-width maximum;
- supersede and retire helpers for reassignment and closure; and
- deterministic first-failure rejection metadata.

Every applied report changes only the fields its kind names. Every rejection
and replay preserves all modeled state fields.

## Proof portfolio

Direct `proof fn` artifacts in `src/fenced_attempt/proofs.rs` cover:

| Theorem | Requirement |
|---|---|
| `current_epoch_required` | r[fenced-attempt-admission.fence-safety] |
| `reassignment_advances_epoch` | r[fenced-attempt-admission.fence-safety] |
| `duplicate_event_is_noop` | r[fenced-attempt-admission.event-idempotence] |
| `event_conflict_rejected` | r[fenced-attempt-admission.event-idempotence] |
| `terminal_phase_closed` | r[fenced-attempt-admission.terminal-closure] |
| `completion_requires_linkage` | r[fenced-attempt-admission.completion-linkage] |
| `rejection_preserves_state` | r[fenced-attempt-admission.rejection-preservation] |
| `exec_refines_spec` | r[fenced-attempt-admission.refinement] |

Each spec/exec pair carries an `ensures` clause; each proof function discharges
its named theorem directly.

## Consumer mapping profile

Product-neutral mapping obligations, documented with the module:

| Modeled fact | Consumer supplies | Fail-closed rule |
|---|---|---|
| Phase | exact variant match | unsupported phase rejects |
| ReportKind | exact variant match | unsupported kind rejects |
| Epoch | equality fact after consumer admission | inexact epoch facts reject |
| JobId / RunId | equality fact | mismatched identity rejects |
| EventId / PayloadId | equality facts per event | unresolved identities reject |
| Scope | worker and output authority grants | denied scope rejects |
| ResultLink | retained run and payload identity | unknown or mismatched link rejects |

Consumers own projection, parity tests, runtime, and evidence policy. Passing
this model does not claim consumer-source equivalence.

## Baseline

The baseline source was
`5965871d734b56d132cde1cdd0e6b25962c11dc4`. Cargo, exact-source Nix Verus,
and Tracey passed before implementation. The baseline had 10,969 verified
obligations and 2,196 covered requirements.

See `evidence/baseline-2026-09-03/`.

## Final Evidence

| Check | Outcome |
|---|---|
| Focused Cargo tests | 17 passed |
| Focused Verus | 23 verified, 0 errors |
| Full Trellis wrapper | `=== ALL CHECKS PASSED ===` |
| Full metric check | 10,992 Verus proofs; 5,118 tests; 2,204 requirements; 619 modules; 16 trusted boundaries |
| Strict Clippy | passed with `-D warnings` |
| Tracey uncovered | 0 of 2,204 |
| Tracey untested | 0 of 2,204 |
| Octet | 3,867 accepted; 0 new; 0 stale |
| JSON artifact contracts | positive and negative fixtures passed |
| Consumer profile | 39 modules, 42 artifacts, and 10 fixtures passed |
| Cairn strict validation | passed |
| Cairn proposal, design, and tasks gates | passed |

The full wrapper ran inside the repository Nix development shell. A temporary
`.worktrees/aspen` symlink restored the existing `../aspen` sibling relation.
The command removed the link after completion.

See `evidence/final-2026-09-03/`.

## Octet Ratchet

The first Octet run found thirty-one new findings, all inside the new
component. Qualified owner paths, a path-local stance type, positive-predicate
bindings, and an import-free formatter path removed every finding.

The final `src/fenced_attempt/` tree has zero Octet findings. Its three-line
crate-root declaration changed only the existing `src/lib.rs` file-length
identity. The guarded migration replaced that ID and preserved every unchanged
finding record. It added no exception, warning budget, or fenced-attempt
finding.

See `evidence/octet/`.

## Generated and Bound Artifacts

The change refreshed these repository-owned artifacts after source identity
changed:

- `docs/module-index.md`;
- `consumer-profile/generated/profile.json`, from its Nickel source;
- `docs/proof-evidence-bundle.json`; and
- `docs/proof-evidence-ir-chain.json`.

The consumer profile keeps the same 39 UCAN-facing modules. It updates only the
measured crate-root artifact.

## Boundaries

This evidence proves finite decision semantics over admitted facts at the
recorded source and verifier revision. It does not prove:

- transport, storage, or persistence behavior;
- cryptography, signatures, or digest computation;
- clock, worker, sandbox, or scheduling behavior;
- liveness, availability, or durable application by a consumer shell;
- implementation equivalence with any consumer source; or
- verifier soundness.
