## Why

`repair-tinycc-0-9-27-amd64-compile` fixed the minimal TinyCC 0.9.27 compile boundary: trivial `tcc -c hello.c` now succeeds and malformed input exits cleanly. When `repair-make-tcc-amd64-varargs` resumed, GNU make still failed earlier than make runtime smoke: TinyCC 0.9.27 hangs or crashes while compiling make's `getopt.c`.

Debugging showed the deeper predecessor problem: Mes-built `tinycc 0.9.26` emits x86_64 immediate shifts with a zero count. A tiny reproducer compiled by `tinycc 0.9.26` turns `x >> 8` into `shr $0x0,%eax` and `x << 3` into `shl $0x0,%eax`. The archived TinyCC 0.9.27 repair worked around a few byte-emitter shifts, but GNU make sources exercise many more shift paths.

## What Changes

- Repair the Mes-built TinyCC 0.9.26 x86_64 immediate-shift code generation used to build TinyCC 0.9.27.
- Rebuild `bootstrap/tinycc.ncl` from the repaired predecessor without broad source-level shift workarounds for every downstream source.
- Prove TinyCC 0.9.27 can compile both trivial C and GNU make's `getopt.c` to objects.

## Scope

- **In scope**: `bootstrap/tinycc-mes.ncl`, the x86_64 immediate-shift emitter in TinyCC 0.9.26, and a documented narrow TinyCC 0.9.27 follow-up patch only when the repaired predecessor exposes a specific remaining varargs/path crash such as relocation-section name formatting.
- **Out of scope**: completing GNU make runtime smoke, musl, or binutils. Those stay in their parent changes after this compiler boundary passes.

## Evidence Needed

Completion requires a source-pin audit, a positive shift-codegen object disassembly showing nonzero shift immediates, successful `crunch build bootstrap/tinycc.ncl`, `tcc -c hello.c`, `tcc -c getopt.c`, malformed-input clean failure, and a host-leakage scan that rejects undeclared `/usr/bin`, host compiler/binutils/libc paths, Nix commands, and undeclared `/nix/store/*-{gcc,binutils,glibc}` references.

## Parent

Discovered by `repair-make-tcc-amd64-varargs` task I3 while trying to build `bootstrap/make-tcc.ncl` after the trivial TinyCC compile repair.
