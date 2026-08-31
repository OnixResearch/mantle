# V87 native-binding-root promoted proof launch

## Status

V87 launched as a detached promoted proof on Leviathan. This launch record does
not claim proof completion.

## Bound inputs

- Source commit: `a1537652b6068b415d65f1e85527515409ccc453`
- Orchestrator BLAKE3:
  `eb8d5919c9a5298a035f89fc3894bc82c81be31c9c016158902f0d64ff647027`
- Ready source-profile BLAKE3:
  `68a074196a7348fecc0abad64b3f8cd034126d33e66305cff2a389297a014894`
- Expected StageX lineage BLAKE3:
  `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`
- Expected native-provider BLAKE3:
  `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Disk bound: 700,000,000,000 bytes
- Observed free bytes before execution: 738,716,893,184

## Detached process identity

- Wrapper PID: `3352131`
- `/proc` start ticks: `71280663`
- Launch time: `2026-08-31T01:11:25-04:00`
- Watcher: local pueue task `582`

The launcher detached the wrapper from SSH and pueue standard input and output.
The watcher checks both PID liveness and `/proc` start ticks.

## Preserved launch evidence

This directory contains the source transfer record, source-profile refresh and
verification, detached process records, host facts, initial log snapshot, and
all V87 operator scripts.

## Non-claims

This record does not prove Rust-provider completion, checkpoint publication,
stage1 or stage2 success, fixed-point equality, final receipt validity, or
`complete` trust status.

## Next action

Monitor the exact detached process. If it fails, preserve its staging evidence
before repair or cleanup. If it succeeds, verify checkpoint publication,
stage equality, the final receipt, and the operator trust report.
