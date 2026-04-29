Task-ID: I2
Covers: bootstrap.part.tinycc.0.9.27

# tinycc 0.9.27 derivation audit

Checked file: `bootstrap/tinycc.ncl`.

Result: PARTIAL. The derivation has the right dependency order and fixed source pin, but it is not yet a faithful upstream mirror.

Matches upstream:

- Imports `stage0-posix.ncl`, `mes.ncl`, and `tinycc-mes.ncl` before building tcc 0.9.27.
- Fetches upstream `tcc-0.9.27.tar.bz2` as a fixed-output `fetchTarball`.
- Compiles tcc 0.9.27 with `tcc-0.9.26`.
- Runs one self-hosting rebuild (`tcc-pass1` compiles final `tcc`).
- Installs `bin/tcc` and `bin/tcc-0.9.27`, and forwards Mes libc/headers for downstream stages.

Intentional / current Crunch deviations:

- Uses amd64/x86_64 compile defines (`TCC_TARGET_X86_64`) while upstream tcc-0.9.27 script shown in `pass1.kaem` uses i386/x86 paths.
- Does not currently mirror upstream `tcctools.c` file-open patches.
- Does not currently mirror upstream `tccelf.c` `fiwix-paddr` and `check-reloc-null` patches.
- Does not rebuild Mes libc with the new tcc 0.9.27; it copies Mes output from the input `mes.ncl` instead.
- Uses `$out` layout instead of live-bootstrap `/usr` layout.

Carry-forward fix scope for I3/V2:

- Decide whether these deviations are intentional for the current amd64 Crunch chain or implement the missing patch/runtime-rebuild steps before claiming self-contained output-contract validation.
