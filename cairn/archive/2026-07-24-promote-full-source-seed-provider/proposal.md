## Why

Mantle's selected bootstrap seed is still the reduced musl.cc provider even though `bootstrap/seed-full.ncl` describes a source-chain-shaped replacement. The candidate is not promotable: GCC 4.0 installs pass1 bridge objects and wrappers, later compiler/libc/binutils receipts are contract-only, and no committed runtime proof binds a real normalized provider to the self-build fixed point.

Keeping that distinction only in comments makes the highest-risk bootstrap boundary easy to overstate. Mantle needs an executable, fail-closed promotion boundary that accepts only real source-built compiler and runtime artifacts and leaves the legacy selector unchanged whenever any predecessor is bridged, stubbed, host-fallback, or unproven.

## What Changes

- Replace the GCC 4.0 pass1 bridge path with an upstream-faithful source build using the already declared TinyCC/musl/autotools closure.
- Build and smoke the complete GCC 4.0 → GCC 4.7 → GCC 10 → musl 1.2.5/binutils 2.41 provider chain without `seed-legacy.ncl` or musl.cc in the evaluated dependency closure.
- Emit authenticated provider metadata and runtime evidence for compiler drivers, compiler internals, binutils, headers, CRT objects, `libgcc_s.so.1`, and `libc.so`.
- Select the source-built provider only after runtime admission succeeds, then rerun the authenticated fresh-clone fixed-point proof.
- Preserve explicit non-claims about compiler correctness, seed correctness, independent rebuild agreement, and release reproducibility.

## Impact

- **Files**: `bootstrap/{gcc-4.0,seed-full,seed}.ncl`, dependent bootstrap stages as required by observed runtime blockers, provider evidence, focused validation code/tests, README/operator docs, and lifecycle evidence.
- **Testing**: baseline frontier capture; positive and negative no-bridge/no-legacy admission tests; real provider build and tool smoke; source-closure audit; authenticated fresh-clone stage0/stage2 fixed-point proof; first-party quality, machine contracts, Nix evaluation, Cairn gates, and Tracey coverage.
