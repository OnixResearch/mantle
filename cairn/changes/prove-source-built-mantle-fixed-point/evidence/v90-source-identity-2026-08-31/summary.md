# V90 Rust source-identity proof launch

## Status

V90 launched as a detached promoted proof on Leviathan. This record does not
claim completion.

## Bound inputs

- Source commit: `8346c02dd9e73734a821dad04238fe24f88c006d`
- Orchestrator BLAKE3:
  `05d37e395d503a6f56a8c779cda122b1cd0095e54decf18beb95a3480003469a`
- Ready source-profile BLAKE3:
  `575ea7f4a743c48db0896a5d6e00c52823020b0d18dd2a02119bfe449884bf8d`
- Expected StageX lineage BLAKE3:
  `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`
- Expected native-provider BLAKE3:
  `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 701,845,839,872

## Detached process identity

- Wrapper PID: `3392475`
- `/proc` start ticks: `74044974`
- Launch time: `2026-08-31T08:52:09-04:00`
- Watcher: local pueue task `880`

The launcher detached the proof from SSH and pueue standard input and output.
The watcher checks PID liveness and `/proc` start ticks.

## Expected route

V90 must restore the immutable 17-payload checkpoint without repeating the Rust
provider builds. It must pass closure relocation and the bound-loader rustc
route.

Stage1 must BLAKE3-frame typed path, Cargo, and Git source identities before
constructing Rust child-action authority.

## Non-claims

This record does not prove stage1 execution, fixed-point equality, the final
receipt, or complete trust.

## Next action

Monitor the exact detached process. On failure, preserve its staging evidence
before repair or cleanup. On success, verify stage equality, the final receipt,
and the operator trust report.
