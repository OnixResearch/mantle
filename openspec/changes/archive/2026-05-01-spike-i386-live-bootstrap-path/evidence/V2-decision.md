# V2 Decision: keep i386 as a repair/proof track, not production pivot yet

Task-ID: V2

Covers: `bootstrap.i386-live-bootstrap-spike.decision`

## Decision

Do **not** pivot Crunch's production Make 3.82 runtime-validation path to i386-first yet.

Keep both paths explicit:

1. **amd64 path:** remains blocked at generated `make-3.82-tcc` startup/runtime segfaults. Do not add more Make-only patches without lower-level compiler/runtime evidence.
2. **i386 path:** remains promising because native i386 execution under bwrap is proven, but it is not ready for production because the first TinyCC handoff proof hits a compiler-output blocker before producing the no-libc i386 smoke executable.

## Evidence

- `bootstrap/spike-i386-native-smoke.ncl` passes and proves the kernel/bwrap execution model.
- `bootstrap/spike-i386-tinycc26-cross-smoke.ncl` builds `tcc26-i386` and verifies it reports `tcc version 0.9.26-i386-spike (i386 Linux)`.
- The same proof target then segfaults during `./tcc26-i386 -nostdlib -static ... exit42.s`, before a runnable i386 output is produced.

## Next change recommendation

Open or continue a focused i386 TinyCC proof/repair change before touching production `bootstrap/make-tcc.ncl`:

- split `tcc26-i386` smoke into assemble-only, link-only, and run-output substeps;
- capture whether the segfault occurs in source parsing, assembly, final ELF writing, or output execution;
- only after `tcc26-i386` emits and runs the no-libc exit42 ELF should Crunch attempt the StageX/live-bootstrap `tcc-0.9.27 -> make-3.82 pass1` i386 chain.
