# Trellis remote-admission evidence

Mantle compares its remote-attempt core with a finite Trellis model. The comparison applies only to supported normalized cases.

Ordinary runtime admission still calls `plan_remote_attempt_report`. Runtime code does not read Trellis, Kamacite, or Valence evidence.

## Pinned Trellis model

The evidence binds these Trellis facts:

- Repository: `github.com/OnixResearch/trellis`.
- Revision: `8de4b24aa2d66cc2e6ec966d686df023492265d3`.
- Source scope: `src/fenced_attempt`.
- Git tree OID: `91dace31060cf5195beb72dba71640d9091e67bf`.
- Deterministic `git archive` BLAKE3: `e13e9f71da4964ab4d4f04d9c29525b20f0e778997da5ada7b6caa0f1711f56c`.
- Archive size: 61,440 bytes.

The archived Trellis change is `.cairn/archive/2026-09-03-add-fenced-attempt-admission-primitives/`.

The focused Trellis run reported `verification results:: 23 verified, 0 errors`. Its focused runtime suite reported 17 passed tests.

The full proof-gap inventory reported 16 `external_body` sites outside `src/fenced_attempt`. It reported zero other accepted gap markers.

## Modeled fields

Trellis state contains these fields:

- `job`.
- `run`.
- `epoch`.
- `phase`.
- `progress`.
- `events`.
- `result`.

A Trellis report contains these fields:

- `job`.
- `run`.
- `epoch`.
- `event`.
- `payload`.
- `kind`.
- `worker`.
- `output`.
- `result`.

Mantle projects admitted strings and digests into equality classes. The projection does not treat a numeric model identity as a cryptographic digest.

## Mantle mapping

The phase mapping is exact for all seven variants:

| Mantle | Trellis |
|---|---|
| `Queued` | `Queued` |
| `Running` | `Running` |
| `Transferring` | `Transferring` |
| `FinishedUndelivered` | `ResultReady` |
| `Completed` | `Completed` |
| `Failed` | `Failed` |
| `Superseded` | `Superseded` |

The report mapping keeps each Mantle variant visible:

| Mantle | Trellis |
|---|---|
| `Start` | `Start` |
| `Heartbeat` | `Progress` |
| `LogAppend` | `Progress` |
| `TransferCheckpoint` | `Checkpoint` |
| `ResultReady` | `ResultReady` |
| `Failure` | `Failure` |
| `Completion` | `Completion` |

The projection maps fences to `current`, `stale`, `future`, `wrong-job`, or `wrong-run`. It maps events to `new`, `replayed`, or `conflict`.

The projection maps worker and output facts to `granted` or `denied`. It maps retained results to explicit linkage classes.

`progress_events_applied` records the Trellis-compatible progress count. Missing legacy values decode as zero.

## Fail-closed differences

Mantle and Trellis do not have identical semantics for every input combination. The projection rejects these differences before proof coverage:

- Different authority-check order.
- Different authority meaning for `Failure` and `ResultReady`.
- Different history-bound check order.
- Different phase and report transitions.
- Different result retention after a result-ready failure.

The complete matrix has 6,720 cases. It accepts 5,882 cases and rejects 838 unsupported cases.

The unsupported counts are:

- `authority-meaning-mismatch`: 42.
- `authority-order-mismatch`: 708.
- `history-order-mismatch`: 66.
- `phase-report-mismatch`: 20.
- `result-retention-mismatch`: 2.

The matrix does not hide these cases as successful parity. Each unsupported case has a stable reason.

## Evidence chain

The evidence chain uses the accepted Kamacite v1 and Valence contracts.

Kamacite revision `de710a092d351e829abfb288d46124e2db8e5b7f` creates a `formal-proof-candidate` envelope.

Valence revision `27b8b2124e12b80718ded124274fec98bed7a581` validates the candidate with role `property`.

The Valence outcome is `accepted_formal_proof`. The Valence report has no issues.

The artifacts are under `evidence/trellis/remote-admission-v1/`. The aggregate machine report is `report.json`.

The evidence checker validates artifact bytes, logical receipt identity, source identity, roles, assumptions, claims, and runtime separation.

## Validation

Run the focused matrix:

```text
cargo test -p crunch-build --lib remote_attempt_trellis --offline
```

Validate the evidence and mutation fixtures:

```text
cargo -Zscript --offline tools/check-trellis-remote-admission.rs --self-test
```

If the exact Trellis checkout is available, validate its source archive:

```text
cargo -Zscript --offline tools/check-trellis-remote-admission.rs \
  --self-test \
  --trellis-root /path/to/trellis-at-8de4b24
```

Validate the Nickel policy:

```text
nickel typecheck config/trellis-remote-admission.ncl
```

Validate the machine contract:

```text
cargo -Zscript --offline scripts/check-machine-schema-contracts.rs
```

## Update procedure

1. Change the Trellis model through a separate accepted Trellis change.
2. Pin the new Trellis revision and source archive identity.
3. Regenerate the oracle with the exact Trellis executable model.
4. Review every new supported or unsupported case.
5. Regenerate the Kamacite envelope and Valence receipt.
6. Regenerate `report.json` and the machine-contract fixtures.
7. Run all positive, negative, mutation, source, and lifecycle checks.
8. Report the old proof as stale until all identities agree.

Do not update only the recorded digest. A source, model, fixture, policy, role, or assumption change requires complete regeneration.

## Claim boundary

The accepted claim covers the named abstract safety properties for the recorded supported projection cases.

The evidence does not prove:

- Full implementation equivalence.
- Persistence atomicity.
- Transport reliability.
- Cryptographic correctness.
- Remote worker correctness.
- Liveness or availability.
- Whole-build correctness.
- Release eligibility.
