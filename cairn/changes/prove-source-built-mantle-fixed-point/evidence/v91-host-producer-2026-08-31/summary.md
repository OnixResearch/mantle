# V91 consumed-host producer proof launch

## Status

V91 launched as a detached promoted proof on Leviathan. This record does not
claim completion.

## Bound inputs

- Source commit: `22befa5ad697063939279709f715fa3fe4b3b1bf`
- Orchestrator BLAKE3:
  `1a8829e3fedf133585a54759a65f72ba410ba5382ae43ece44dacf388c33c52b`
- Ready source-profile BLAKE3:
  `8f044865fdcfe1ee15d00a56b405038d76b6746368c363cca41722185a413c61`
- Expected StageX lineage BLAKE3:
  `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`
- Expected native-provider BLAKE3:
  `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 702,941,278,208

## Detached process identity

- Wrapper PID: `3398763`
- `/proc` start ticks: `74421071`
- Launch time: `2026-08-31T09:54:50-04:00`
- Watcher: local pueue task `1012`

The launcher detached the proof from SSH and pueue standard input and output.
The watcher checks PID liveness and `/proc` start ticks.

## Expected route

V91 must restore the immutable checkpoint, pass closure and rustc runtime
checks, and retain BLAKE3-framed source identities.

Stage1 must resolve Cargo-omitted proc-macro producers only through the unit's
unique consumed host artifact.

## Non-claims

This record does not prove stage1 execution, fixed-point equality, the final
receipt, or complete trust.

## Next action

Monitor the exact detached process. On failure, preserve its staging evidence
before repair or cleanup. On success, verify stage equality, the final receipt,
and the operator trust report.
