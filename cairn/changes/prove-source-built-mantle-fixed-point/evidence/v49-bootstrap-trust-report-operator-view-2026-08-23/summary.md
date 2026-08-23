# V49 bootstrap trust-report operator view

## Question

Can an operator revalidate V48 and see its exact bounded trust status without turning missing child-action authority into a stronger claim?

## Inspected evidence

Commit `b77d4647` adds a functional-core and imperative-shell trust report. The shell revalidates the v2 deterministic receipt, durable proof-bundle digest, fixed-point plan, stage evidence, and optional action-trust links. The pure core emits the complete claim only when both action files reconcile without unknown, missing, remote, cache-only, fallback, or authority events.

The final release binary has BLAKE3 `12966408c0223e95783b864681e93f5715e1358f2954fc36510a534b71f3d9ca`. Local and Leviathan hashes match.

The command ran against the complete V48 proof root on Leviathan:

```text
mantle --json bootstrap trust-report \
  --proof-root /home/brittonr/mantle-runs/receipt-fix-v31/source-built-fixed-point-v48-final-receipt-20260823
```

The command completed in 10.71 seconds. It independently reported:

- schema `mantle-bootstrap-trust-report-v1`;
- status `fixed-point-only`;
- `fixed_point_verified: true`;
- `root_action_trust_complete: false`;
- receipt BLAKE3 `698e0eea069c7ba2fbe7531d3dea214a841d63e33d85a508545fad21cf71193e`;
- plan BLAKE3 `fc61749204d950065ae1c0c40a9272a64040a7a4365e4e60d7bb25ea6fb5e464`;
- proof-bundle BLAKE3 `c97ff46ff25df813bd893dec84fd7f01244d0eabc241e53ae9ae5388a61011c2`;
- six planned and observed stages;
- two current executions and four restored checkpoint stages;
- zero substitutions, stage authority violations, and fallback events.

The report preserves three blockers:

- `action-trust-plan-not-bound`;
- `execution-reconciliation-not-bound`;
- `complete-child-action-coverage-not-proven`.

Focused evidence records 10 passing receipt tests, 10 passing trust-report core and shell tests, and 3 passing CLI tests. The negative tests reject partial action links, digest drift, linked-evidence tampering, symlinked action files, remote events, cache-only completion, unknown events, missing actions, count overflow, fallback, and stage-count drift.

The focused Clippy gate passed with `-D warnings`. It retained narrow command-line allowances for the unrelated existing `nix_free_demo_cmd` large enum, `rust_plan` `chunks_exact` finding, and root dead-code warnings. Nickel type checking, the 177-command operator contract, and `git diff --check` passed.

## Decision

Accept the operator view as a verified bounded report. Do not mark the root action-trust requirement complete.

The command closes the missing CLI surface and replaces the earlier Clap error with an exact report. It does not backfill authority that V48 never bound.

## Owner

`source_built_trust_report` owns pure claim classification. `source_built_trust_report_shell` owns proof-root reads and receipt revalidation. The fixed-point receipt remains the authority source.

## Next action

Define and emit complete child-action adapters before execution. Bind the root action-plan and reconciliation file digests into a new promoted receipt, then rerun the proof and this report.

## Non-claims

This view does not prove compiler correctness, seed correctness, kernel isolation, independent rebuild agreement, release reproducibility, deployment success, or full Cargo compatibility.
