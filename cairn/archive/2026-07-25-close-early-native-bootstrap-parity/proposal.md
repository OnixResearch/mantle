## Why

The current bootstrap parity report still marks `binutils.tcc` and `gcc.4.0` partial. Existing receipts prove bounded tool and compiler slices, while the admitted provider proves later runtime surfaces, but neither establishes the exact early native-lineage contract demanded by parity. Static status changes would conceal the gap: Mantle must either produce the missing native artifacts from authenticated sources or retain the blocker.

These two rows form one causal boundary. The TCC-era binutils handoff supplies the assembler/linker/archive tools used by the regenerated GCC 4.0 build, and GCC 4.0 must produce real compiler, generator, demangler, libgcc, and C++ artifacts without TinyCC delegation or fabricated objects before later GCC rows can be promoted.

## What Changes

- Derive an executable completion contract for the `binutils.tcc` and `gcc.4.0` parity rows from current outputs and source recipes.
- Replace remaining omitted-member, compatibility-bridge, release-generated, placeholder, wrapper-delegated, or state-pinned artifacts in the claimed early lineage.
- Add BLAKE3-bound receipts and positive/negative semantic rails for the early binutils and GCC outputs.
- Promote each row only from current runtime evidence; keep the other row blocked if its own contract is incomplete.

## Dependencies

- The accepted bootstrap source inventory, source-pin audit, and GCC 4.0 configure-preprocess bridge confinement remain inputs.
- This change is independent of `bind-full-source-rust-provider` and may proceed in parallel.

## Impact

- **Files**: `bootstrap/{binutils-tcc,gcc-4.0-native,gcc-4.0}.ncl`, source regeneration inputs, parity checker and receipts, focused scripts/tests, documentation, and lifecycle evidence.
- **Testing**: source construction, positive tool/compiler behavior, malformed-input rejection, mutation-style bridge detection, parity CLI, source pins, Cairn gates.