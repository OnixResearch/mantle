# Real i386 Mes libc and explicit tcc27 link evidence

Task-ID: repair-i386-tcc27-preprocessor-tccgen task 7
Covers: Build/import a real i386 Mes `libc.a` and rerun the explicit `-nostdlib` tcc27 object link before returning to Make 3.82.

## Changes

- `bootstrap/spike-i386-mes-runtime-layout.ncl` now builds a real i386 Mes runtime archive from individual Mes libc objects instead of carrying the previous placeholder `libc.a`.
- The three i386 Mes syscall C files that crash the predecessor compiler are skipped and replaced with a tiny i386 assembly syscall/helper object.
- The syscall/helper object also provides the minimal libgcc-style conversion helpers needed by the TinyCC 0.9.27 object link (`__fixunsxfdi`, `__fixdfdi`, `__fixsfdi`, `__fixxfdi`, `__floatundixf`).
- `tcc26-i386 -ar` and BusyBox `ar` were not usable for the larger archive, so the spike writes a deterministic SysV archive directly with BusyBox-prefixed file commands.
- `bootstrap/spike-i386-tinycc26-cross-smoke.ncl` now preserves useful undefined-symbol diagnostics from the predecessor linker instead of printing literal `%s` for that failure class.
- The gating `tcc27_link` now uses the proven explicit `-nostdlib` object link against `crt1.o`, `tcc27.o`, `libtcc1.o`, and the real libc object set. The canonical source/link path remains a non-gating diagnostic because it still routes through `/no-runtime` startup/library search and segfaults.

## Verification

Command:

```sh
./target/debug/crunch eval bootstrap/spike-i386-mes-runtime-layout.ncl >/tmp/spike-eval.json && \
rm -rf .crunch-drain/i386-final1-store && \
mkdir -p .crunch-drain/i386-final1-store && \
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/i386-final1-store" \
  bootstrap/spike-i386-mes-runtime-layout.ncl \
  > openspec/changes/repair-i386-tcc27-preprocessor-tccgen/evidence/build-final1.stdout.log \
  2> openspec/changes/repair-i386-tcc27-preprocessor-tccgen/evidence/build-final1.stderr.log
```

Exit status: `0` (`build-final1.rc`).

Key builder step results from the output bundle:

- `runtime_libc_archive`: `0`
- `tcc27_compile_object`: `0`
- `tcc27_link`: `0`
- `tcc27_version`: `0`, stdout: `tcc version 0.9.27-i386-spike (i386 Linux)`
- `make_compile_getopt`: `0`
- `tcc27_link_canonical_diagnostic`: `139`, expected non-gating diagnostic showing the old `/no-runtime` search path still fails.

Output summary: `status=partial`, `blocked_step=make_full_smoke_not_yet_implemented`.

## Next blocker

The requested real-libc explicit object link is repaired. The next Make 3.82 slice should promote this from the spike into the production handoff and then implement a fuller Make smoke beyond compiling `getopt.c`.
