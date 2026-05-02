# i386 tcc27 link diagnostic 4

Task-ID: repair-i386-tcc27-preprocessor-tccgen
Covers: post-compile i386 TinyCC 0.9.27 link narrowing after the native-387 compile repair.

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/i386-link-diag4-store" \
  bootstrap/spike-i386-mes-runtime-layout.ncl
```

Exit status: `0`

Output path:

```text
/home/brittonr/git/crunch/crunch/.crunch-drain/i386-link-diag4-store/yi42jhd59h4d6csfp8p5b39mddmyay0b-spike-i386-mes-runtime-layout
```

Copied logs: `evidence/logs-link-diag4/`

## Result

This slice moved the post-compile blocker forward and made the failure concrete:

- `tcc27_compile_object.rc`: `0`
- `tcc27_object_exists.rc`: `0`
- `tcc27_link_object_nostdlib_crt1.rc`: `1` with no predecessor missing-file diagnostic
- `tcc27_link_object_B_L.rc`: `139`
- `tcc27_link_source_B_L.rc`: `139`
- `tcc27_link.rc`: `139`

Key narrowing:

1. The previous `tcc27_compile_object.rc=0` was incomplete evidence: with `-o tcc27.o` before the source, the Mes-built `tcc26-i386` reported success but did not leave `tcc27.o` behind. Moving `-o tcc27.o` after `tcc.c` and adding `test -f tcc27.o` proves the full ONE_SOURCE object is now actually materialized.
2. Rebuilding the predecessor with direct, non-varargs missing-file diagnostics showed the default source link is still using the predecessor's compile-time `/no-runtime` CRT/libc defaults; `-B`/`-L` only partially changes the search path.
3. The explicit `-nostdlib` object link no longer hits startup/library search diagnostics, but still exits `1`. The remaining blocker is now real runtime/archive content rather than tccgen/source emission: the sibling layout still uses the earlier placeholder `libc.a` copied from `libtcc1.a`, so it cannot yet produce a complete tcc27 executable.

## Selected diagnostics

```text
tcc27_compile_object.rc=0
tcc27_link.rc=139
  /no-runtime/crt1.o
  file not found: crt1.o
  /no-runtime/libc.a
  /no-runtime/tcc/libc.a
  library not found: c
tcc27_link_object_B_L.rc=139
  /no-runtime/crt1.o
  file not found: crt1.o
  /tmp/spike-i386-mes-runtime-layout/i386-mes-runtime/lib/mes/tcc/libc.a
  /no-runtime/libc.a
  /no-runtime/tcc/libc.a
tcc27_link_object_nostdlib_crt1.rc=1
tcc27_link_source_B_L.rc=139
  /no-runtime/crt1.o
  file not found: crt1.o
  /tmp/spike-i386-mes-runtime-layout/i386-mes-runtime/lib/mes/tcc/libc.a
  /no-runtime/libc.a
  /no-runtime/tcc/libc.a
tcc27_object_exists.rc=0
```

## Next blocker

Build or import a real i386 Mes `libc.a` for the sibling runtime layout, then rerun the explicit `-nostdlib` object link first. Do not return to Make 3.82 until `tcc27_link_object_nostdlib_crt1` and the canonical `tcc27_link` either produce an executable or narrow to a true TinyCC relocation/runtime bug.
