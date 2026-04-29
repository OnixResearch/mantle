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
3. Test the smallest source-normalization or mescc setup change that removes the `BufferedFile` diagnostics.
4. Re-run `crunch build bootstrap/tinycc-mes.ncl` and require `tcc-boot0` to complete.

## Validation

- `crunch build bootstrap/tinycc-mes.ncl --no-substitute -j 1 --verbose --log-level info` succeeds.
- Smoke executes `bin/tcc --version` and compiles a trivial C program with the produced compiler.
