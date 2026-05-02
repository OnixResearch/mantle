# i386 tcc27 preprocessor/tccgen diagnostic summary

Task-ID: repair-i386-tcc27-preprocessor-tccgen
Covers: failed source-emission patch rollback, focused tccgen reduction probe, next blocker narrowing

## Change

`bootstrap/spike-i386-mes-runtime-layout.ncl` now keeps the TinyCC 0.9.27 source intact with respect to `tccpp.c::pp_line()`: the prior attempted line-marker rewrite was removed because the `tcc27_preprocess_line_markers` crash comes from the already-built predecessor `tcc26-i386` while it preprocesses input, not from the tcc27 source being compiled.

The sibling proof now adds two non-gating reduction probes in a copied source tree:

- `tcc27_decl_initializer_stub_tccgen`: compiles `tccgen.c` after wrapping the body of `decl_initializer(...)` in `#if 0`.
- `tcc27_decl_initializer_stub_full`: compiles the full `ONE_SOURCE=1` source with the same reduction.

These probes test whether the previously suspected `decl_initializer(...)` body is sufficient to explain the current derivation failure without weakening the main/gating tcc27 source path.

## Verification command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  bootstrap/spike-i386-mes-runtime-layout.ncl \
  > openspec/changes/repair-i386-tcc27-preprocessor-tccgen/evidence/build-default.stdout.log \
  2> openspec/changes/repair-i386-tcc27-preprocessor-tccgen/evidence/build-default.stderr.log
```

Exit status: `0`

Output path: `/nix/store/mzmagid1gv9870h354y668hkn1394h2l-spike-i386-mes-runtime-layout`

Extracted evidence logs are in `evidence/logs/`. The large successful `tcc27_preprocess_no_lines.stdout` transcript is intentionally not committed; its return code is preserved and the build output remains reproducible from the store path above.

## Results

The derivation builds successfully, but the handoff remains blocked:

- `runtime_libtcc1_object.rc`: `0`
- `runtime_libtcc1_archive.rc`: `0`
- `tcc27_preprocess_no_lines.rc`: `0`
- `tcc27_compile_tccpp_unit.rc`: `0`
- `tcc27_compile_tccelf_unit.rc`: `0`
- `tcc27_compile_i386_gen_unit.rc`: `0`
- `tcc27_compile_libtcc_unit.rc`: `0`
- `tcc27_preprocess_line_markers.rc`: `139`, stdout still stops at `# 1 "`
- `tcc27_compile_tccgen_unit.rc`: `139`
- `tcc27_decl_initializer_stub_tccgen.rc`: `139`
- `tcc27_decl_initializer_stub_full.rc`: `139`
- `tcc27_compile_object.rc`: `139`

The reduction result disproves the narrow hypothesis that simply bypassing `decl_initializer(...)` is sufficient in the current derivation. The remaining crash is earlier or broader in the predecessor compiler path: likely a tcc26-i386/Mes varargs or string diagnostic path reached while parsing/compiling tccgen/full-source input, or another nearby tccgen construct that the line-number stub does not bypass.

## Next blocker

Continue with a predecessor-focused reduction rather than tcc27 `pp_line()` source edits:

1. Generate smaller `tccgen.c` cut-down diagnostics in the sibling proof (function-range cuts or preprocessed-source chunks) until a non-segfaulting boundary is found.
2. Use gdb/objdump evidence from restored `tcc26-i386`: current local reproduction crashes in the predecessor's `strlen`/formatting path with a bogus pointer while compiling `tccgen.c`, which supports a diagnostic/varargs corruption hypothesis.
3. Only promote a source patch to the gating tcc27 compile once `tcc27_compile_object.rc` moves from `139` to `0`.
