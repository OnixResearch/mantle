# Proposal: Verify Wasm source-closure trees

## Why

Molten's private-snapshot intake exposed a provider contract defect: an unchanged
Mantle pipeline bundle declares a directory-valued source closure, and producer
remeasurement accepts that directory, but the published consumer shell marks it
Missing. The shell also retained an ambient root path rather than an opened
capability and buffered regular-file contents. Repair the published boundary
before pinning and admitting the same bytes into Molten.

## What Changes

- Admit bounded source-closure directories with unchanged producer v1 identity;
  preserve file-valued source closures and every other regular-file role.
- Retain opened directory roots and resolve every descendant no-follow; stream
  files and reject symlink, FIFO, special-node, drift, and bound violations.
- Reuse pinned bounded-tree observation/planning/copy/revalidation and expose
  one source-only capability-rooted copying API for private consumer snapshots.
- Add producer parity, adversarial member/copy, and actual pinned-pipeline
  regressions. Keep report schema, complete member agreement, and non-claims.

## Capabilities

### Modified Capabilities

- `wasm-component-consumer-verifier`: source-tree framing, explicit capability
  roots, source-only copying, bounded compatibility, and pipeline evidence.

## Impact

Implementation stays in `mantle-wasm-consumer-verifier`'s optional std shell.
The pure core and producer identity do not change. Std-only dependencies reuse
workspace cap-std/cap-fs-ext and immutable bounded-tree pins. The real pipeline
regression imports the published facade as a root dev-dependency only. Molten
owns snapshot admission, not Mantle runtime authority. No actor/extension
installation, effect linker, deadline, rollback, runtime cutover, or correctness
claim is included. Mantle's Wasm producer/verifier maintainers own this contract.
