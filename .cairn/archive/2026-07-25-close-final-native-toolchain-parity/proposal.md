## Why

The selected source-built provider passes its bounded admission contract, but the parity report still marks `gcc.4.7`, `gcc.10`, and `full-musl-binutils` partial. Current evidence explicitly notes release-generated sources, state-pinned overlay inputs, and incomplete derivational-closure or native-correctness evidence. Provider admission and parity answer different questions; the stronger rows must not be promoted by copying the admitted-provider result.

The final native handoff must therefore be rebuilt and evidenced as a continuous derivational closure from the completed early GCC boundary through regenerated GCC 4.7 and GCC 10 to final musl and binutils outputs.

## What Changes

- Regenerate and build GCC 4.7 and GCC 10 from authenticated sources using only the preceding admitted compiler/tool closure.
- Build final musl and binutils from that compiler and bind static/shared C/C++ runtimes, CRT, linker, archive, relocation, malformed-input, and copied-tree behavior.
- Remove state-pinned overlay artifacts and impure transitive inputs from the claimed closure.
- Emit independent row receipts and promote only rows whose current artifacts satisfy their complete contracts.

## Dependencies

- `close-early-native-bootstrap-parity` must complete first.

## Impact

- **Files**: `bootstrap/{gcc-4.7,gcc-10,gcc-10-final,musl-full,binutils-full,seed-full}.ncl`, regenerated-source recipes, parity evidence/checkers, tests, documentation, and lifecycle evidence.
- **Testing**: source regeneration, compiler/runtime/binutils behavior, copied-tree relocation, malformed-input rejection, closure scans, source pins, parity CLI, Cairn gates.