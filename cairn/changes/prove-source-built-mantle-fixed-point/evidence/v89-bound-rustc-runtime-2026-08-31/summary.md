# V89 bound-rustc-runtime proof launch

## Status

V89 launched as a detached promoted proof on Leviathan. This record does not
claim completion.

## Bound inputs

- Source commit: `bf2abe26065d9316d983a829735d32de7c5e2db3`
- Orchestrator BLAKE3:
  `1b320d7e01124dbee258e6f8c7d94d1fa9d79ba9a6cd8818e66f85fce2238ac3`
- Ready source-profile BLAKE3:
  `d3cb7e46c2386b9c3c30245e0bd322a252f739c66136f2c271ce1a64c7b59514`
- Expected StageX lineage BLAKE3:
  `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`
- Expected native-provider BLAKE3:
  `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 710,194,757,632

## Detached process identity

- Wrapper PID: `3388643`
- `/proc` start ticks: `73385511`
- Launch time: `2026-08-31T07:02:14-04:00`
- Watcher: local pueue task `815`

The launcher detached the proof from SSH and pueue standard input and output.
The watcher checks PID liveness and `/proc` start ticks.

## Expected checkpoint and runtime route

V89 must restore immutable checkpoint
`3d6ba9154ac60e3214e8486f8088050157c007667208e2e8970e8e397b1824ca` /
`c9918c0ede2fd774f5348a775981838c5590903ce6ba6a691317a766a052b3eb`.
It must not repeat the five Rust-provider builds.

The proof must launch restored `rustc.dynamic` through the binding-owned musl
loader and C++ runtime. Ambient `LD_LIBRARY_PATH` remains forbidden.

## Non-claims

This record does not prove checkpoint restoration, rustc compatibility, stage
execution, fixed-point equality, final receipt validity, or complete trust.

## Next action

Monitor the exact detached process. On failure, preserve its staging evidence
before repair or cleanup. On success, verify stage equality, the final receipt,
and the operator trust report.
