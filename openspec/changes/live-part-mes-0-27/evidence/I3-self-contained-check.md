Task-ID: I3
Covers: bootstrap.part.mes.0.27

# Mes 0.27 self-contained derivation check

Checked file: `bootstrap/mes.ncl`.

Result: PASS. No source edit was needed for this task.

Self-contained facts:

- Sources are declared as fixed-output fetches:
  - `mes-src`: `https://mirrors.kernel.org/gnu/mes/mes-0.27.1.tar.gz`
  - `nyacc-src`: `https://github.com/Googulator/nyacc/releases/download/V1.00.2-lb1/nyacc-1.00.2-lb1.tar.gz`
- Inputs are explicit: `[stage0, mes_src, nyacc_src]`.
- Runtime tool boundary is explicit: builder is `/bin/sh`, with `/bin/busybox` supplied by the sandbox shell and stage tools from `stage0-posix`.
- Patches are in-derivation and deterministic:
  - `kaem.run` base rewrite from `0x1000000` to `0x8048000`.
  - `lib/linux/wait4.c` status-pointer initialization when missing.
  - Mes-safe module merge / preservation for NYACC compatibility.
  - `assert-system*` normalization for mes-m2's pointer-like `system*` success values.
- Output contract is asserted by build-time `test` checks for executable `bin/mes-m2`, populated `lib/x86_64-mes`, headers, and required archive/object files.

Negative space checked:

- No implicit host `git`, `curl`, `tar`, compiler, or Guile dependency is referenced by the builder script.
- Source-pin audit V1 independently verified every fetch block has a hash.
