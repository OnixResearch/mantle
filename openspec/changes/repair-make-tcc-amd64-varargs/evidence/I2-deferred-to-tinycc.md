Task-ID: I2
Covers: bootstrap.part.make.3.82.amd64.execution

# Deferred to TinyCC 0.9.27 compile repair

Result: DEFERRED.

`repair-make-tcc-amd64-varargs` tried both repair strategies allowed by the design:

1. Use the archived `tinycc 0.9.27` handoff directly. It reports version, but bounded `tcc -c hello.c` times out after reading the input and growing the heap repeatedly. `tcc -vv -c hello.c` segfaults after printing Mes-libc-corrupted format output.
2. Use compile-capable `tinycc 0.9.26` and patch GNU make's amd64-unsafe paths. Object compilation succeeds, and fixed-arity `concat2`/`concat3`/`concat4`/`concat5` removes the first observed `concat(...)` segfault. The resulting GNU make still exits `60` for `make -f Makefile` without executing a recipe, even with a minimal environment and no built-in rules.

Conclusion: make repair depends on a predecessor compiler/runtime boundary repair, not another local make-only patch. New scoped change:

- `repair-tinycc-0-9-27-amd64-compile`

This parent repair remains open until that predecessor compiler can compile trivial C and `bootstrap/make-tcc.ncl` can be rebuilt with passing positive/negative Makefile smoke evidence.
