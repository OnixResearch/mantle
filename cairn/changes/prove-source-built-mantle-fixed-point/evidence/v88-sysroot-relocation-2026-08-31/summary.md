# V88 checkpoint sysroot-relocation proof launch

## Status

V88 launched as a detached promoted proof on Leviathan. This launch record does
not claim completion.

## Bound inputs

- Source commit: `79cd761ab2897d6eb362afa0846de90be15b8d9f`
- Orchestrator BLAKE3:
  `1f693b32d06b5e386cc0c1e30c8245c37bd6fdc11d8848129e156d2baa94e549`
- Ready source-profile BLAKE3:
  `951478d5a90c5b8df61dcf62ecf742d6bd39507611582a8de06d0c49b05e544e`
- Expected StageX lineage BLAKE3:
  `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`
- Expected native-provider BLAKE3:
  `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Disk bound: 700,000,000,000 bytes
- Observed free bytes before execution: 725,981,511,680

## Detached process identity

- Wrapper PID: `3356249`
- `/proc` start ticks: `71595352`
- Launch time: `2026-08-31T02:03:52-04:00`
- Watcher: local pueue task `705`

The launcher detached the proof from SSH and pueue standard input and output.
The watcher checks PID liveness and `/proc` start ticks.

## Expected checkpoint

V88 must restore immutable checkpoint
`3d6ba9154ac60e3214e8486f8088050157c007667208e2e8970e8e397b1824ca` /
`c9918c0ede2fd774f5348a775981838c5590903ce6ba6a691317a766a052b3eb`.
It must not repeat the five Rust-provider builds.

## Non-claims

This record does not prove checkpoint restoration, stage execution, fixed-point
equality, final receipt validity, or complete trust.

## Next action

Monitor the exact detached process. On failure, preserve the staging evidence
before any repair or cleanup. On success, verify stage equality, the final
receipt, and the operator trust report.
