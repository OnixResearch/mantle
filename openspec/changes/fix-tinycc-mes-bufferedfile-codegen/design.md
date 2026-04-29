# Design: Fix tinycc Mes `BufferedFile` codegen blocker

## Context

Current `bootstrap/tinycc-mes.ncl` reaches `tcc-mes -version`, then segfaults while compiling `tcc-boot0`. The failing transcript includes:

```text
->type--: not a <type>: (typename "BufferedFile")
rank--: not a pointer:
unexpected size:42
```

The existing derivation already normalizes the `BufferedFile` typedef shape and locally appends an `abort` object to the Mes `libc+tcc.a` link archive, so the next fix must verify the generated `tcc.s`/linked `tcc-mes` semantics rather than only adding output assertions.

## Approach

1. Reproduce the failure from `live-part-tinycc-0-9-26/evidence/V2-build.md`.
2. Compare Crunch edits against live-bootstrap `steps/tcc-0.9.26/pass1.kaem` and required simple patches.
3. Classify the root cause before editing:
   - Missing upstream patch: upstream `pass1.kaem` or `simple-patches/` changes a source file that Crunch does not change yet.
   - Local source normalization: Crunch changes the same source region as upstream but produces different `tcc.s` diagnostics.
   - Mes/mescc setup: source edits match upstream but environment, module path, library path, or Mes runtime archives differ.
4. Test the smallest change in the classified bucket that removes the `BufferedFile` diagnostics.
5. Re-run `crunch build bootstrap/tinycc-mes.ncl` and require `tcc-boot0` to complete.

## Validation

- `crunch build bootstrap/tinycc-mes.ncl --no-substitute -j 1 --verbose --log-level info` succeeds.
- Transcript shows `tcc-boot0` compiles without `Segmentation fault`.
- Output includes executable `bin/tcc` and `bin/tcc-0.9.26`.
- Smoke executes `bin/tcc --version` and compiles a trivial C program with the produced compiler.
- `tcc-mes -version` alone is negative evidence: it may appear in the transcript, but it is never sufficient acceptance evidence for this change.
