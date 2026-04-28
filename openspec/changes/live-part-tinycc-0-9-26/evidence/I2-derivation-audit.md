Task-ID: I2
Covers: bootstrap.part.tinycc.0.9.26

# tinycc 0.9.26 derivation audit

Checked file: `bootstrap/tinycc-mes.ncl`.

Result: PARTIAL / BLOCKED for later build proof. The derivation structure tracks upstream ordering, but the known live-bootstrap tinycc-mes blocker remains: `tcc-mes -version` is not sufficient evidence, and the first self-compile can still fail or segfault when compiling `unified-libc.c` / `tcc-boot0`.

Matches upstream:

- Imports `stage0-posix.ncl` and `mes.ncl` before building tcc 0.9.26.
- Fetches Mes 0.27.1 and Janneke's patched tcc 0.9.26 fork as fixed-output sources.
- Applies the upstream `tcctools.c` file-open patch behavior.
- Emits `tcc.s` via `mes-m2 --no-auto-compile -e main mescc.scm`.
- Links `tcc-mes` with base address `0x08048000` and `-l c+tcc`.
- Rebuilds Mes runtime archives and bootstraps `tcc-boot0`, `tcc-boot1`, `tcc-boot2`, then installs final `tcc`.

Intentional Crunch deviations:

- Uses `$out`-relative install layout instead of live-bootstrap `/usr` paths.
- Recreates source unpacking through fixed-output fetchers instead of `DISTFILES` + `untar` in the build script.
- Adds `normalize_buffered_file_typedef` to avoid a mescc self-referential typedef parse/codegen hazard.
- Adds `append_abort_object` before linking `tcc-mes` so Mes' `libc+tcc.a` has an `abort` symbol.
- Builds a unified Mes libc source in the derivation and uses local helper functions for repeated runtime rebuilds.
- Keeps only x86_64/amd64 behavior for this part; x86 and riscv64 branches from upstream are out of scope for this Crunch output.

Current risk to carry forward:

- The derivation should not be marked fully validated until V2/V3 prove a real `tcc` can build and execute, not just print `tcc-mes -version`.
