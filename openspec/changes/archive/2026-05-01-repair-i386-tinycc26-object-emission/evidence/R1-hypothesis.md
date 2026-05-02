# R1 fault hypothesis

Task-ID: R1
Covers: bootstrap.i386-tinycc26-object-emission.repair

The initial `tcc26-i386` binary was x86_64-hosted and i386-targeting, but it was emitted by Crunch's Mes-built TinyCC 0.9.26 without the source-normalization seams already required by the amd64 handoff.

Evidence from the failed diagnostic showed:

- `tcc26-i386 -version` succeeded.
- `-c` on assembly/C inputs segfaulted in Mes libc string/varargs paths before object creation.
- After disabling the error-path varargs crash, the real failure surfaced as malformed source/token handling (`missing terminating %c character`).

The repair hypothesis was that the i386 sibling proof needed the same Mes/TCC source-normalization used by `bootstrap/tinycc-mes.ncl` / `bootstrap/tinycc.ncl` when a Mes-built TinyCC emits another static compiler:

- avoid deterministic `snprintf`/`sprintf` string constructors in TCC source paths;
- normalize allocator growth expressions known to hit tcc-mes shift/multiply codegen bugs;
- avoid declaration-merge error paths that enter broken varargs diagnostics;
- replace link-time section symbol constructors with direct `strcpy`/`strcat` operations.
