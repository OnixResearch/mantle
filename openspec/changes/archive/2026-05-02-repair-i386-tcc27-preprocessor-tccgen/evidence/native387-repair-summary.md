# i386 tcc27 tccgen repair summary

Task-ID: repair-i386-tcc27-preprocessor-tccgen
Covers: tccgen/full-object segfault repair, next linker blocker narrowing

## Change

`bootstrap/spike-i386-mes-runtime-layout.ncl` now source-normalizes TinyCC 0.9.27 `tccgen.c::init_putv()` by disabling the first `#if defined TCC_IS_NATIVE_387` branch:

```c
#if 0 /* CRUNCH: tcc26-i386 crashes compiling native x87 long-double initializer path */
```

A local reduction against restored `/tmp/tcc26-out/out/bin/tcc26-i386` showed that this native x87 long-double initializer branch was the standalone `tccgen.c` compile crash. The sibling Crunch build confirms the repair in the hermetic derivation.

The prior `decl_initializer(...)` copied-source probes remain as non-gating diagnostics; after the promoted x87 branch normalization, they also pass, confirming the earlier `decl_initializer` hypothesis was downstream noise rather than the root tccgen compile blocker.

## Verification command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  bootstrap/spike-i386-mes-runtime-layout.ncl \
  > openspec/changes/repair-i386-tcc27-preprocessor-tccgen/evidence/build-promote-native387.stdout.log \
  2> openspec/changes/repair-i386-tcc27-preprocessor-tccgen/evidence/build-promote-native387.stderr.log
```

Exit status: `0`

Output path: `/nix/store/d3qwkbdi9sqjmjfhxydj0a4q48h51hq5-spike-i386-mes-runtime-layout`

Extracted evidence logs are in `evidence/logs-promote-native387/`. The large successful `tcc27_preprocess_no_lines.stdout` transcript is intentionally not committed; its return code is preserved and the build output remains reproducible from the store path above.

## Results

The tccgen/full-object handoff is repaired:

- `runtime_libtcc1_object.rc`: `0`
- `runtime_libtcc1_archive.rc`: `0`
- `tcc27_compile_tccpp_unit.rc`: `0`
- `tcc27_compile_tccelf_unit.rc`: `0`
- `tcc27_compile_i386_gen_unit.rc`: `0`
- `tcc27_compile_libtcc_unit.rc`: `0`
- `tcc27_compile_tccgen_unit.rc`: `0`
- `tcc27_compile_object.rc`: `0`
- `tcc27_decl_initializer_stub_tccgen.rc`: `0`
- `tcc27_decl_initializer_stub_full.rc`: `0`

The next blocker moved forward to link/startup resolution:

- `tcc27_link.rc`: `139`
- stderr:

```text
tcc: error: file '%s' not found
tcc: error: file '%s' not found
tcc: error: library '%s' not found
Segmentation fault (core dumped)
```

` tcc27_preprocess_line_markers.rc` still returns `139`, but this remains a predecessor `tcc26-i386 -E` diagnostic and is no longer on the gating compile path.

## Next blocker

Continue from `tcc27_link.rc=139`. The likely high-ROI seam is startup/library search for the i386 Mes runtime layout (`crt1.o`, empty `crti.o`/`crtn.o`, `libtcc1.a`, `libc.a`) and the predecessor's varargs-broken missing-file diagnostics. Add an explicit link diagnostic matrix using direct startup/archive paths before attempting Make 3.82.
