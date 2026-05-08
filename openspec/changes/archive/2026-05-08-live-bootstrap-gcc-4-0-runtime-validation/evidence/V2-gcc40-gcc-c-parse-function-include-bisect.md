# GCC 4.0 c-parse function-after-include bisect

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: scoped

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

## Change

Added `make_cparse_function_after_includes_config_undef6()` and a progressive include chain from `system.h` through `c-common.h`. Each probe emits the same trivial function body after the selected headers:

```c
void cparse_function_probe (void) { }
int cparse_function_after_includes_probe;
```

The matrix is intended to distinguish "function definition after any GCC header" from "function definition after the full c-parse include stack" before spending more iterations on generated Bison body movement.

## Commands

```sh
./target/debug/crunch --json   --store "$PWD/.crunch-drain/gcc40-cparse-function-include-bisect-store"   --state-dir "$PWD/.crunch-drain/gcc40-cparse-function-include-bisect-state"   build --plan --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl   > evidence/gcc40-cparse-function-include-bisect-20260507/plan.stdout.log   2> evidence/gcc40-cparse-function-include-bisect-20260507/plan.stderr.log

./target/debug/crunch bootstrap validate --resume --json --jobs 1   --store "$PWD/.crunch-drain/gcc40-cparse-function-include-bisect-store"   --state-dir "$PWD/.crunch-drain/gcc40-cparse-function-include-bisect-state"   --evidence-dir evidence/gcc40-cparse-function-include-bisect-20260507   bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Evidence bundle: `evidence/gcc40-cparse-function-include-bisect-20260507/`

## Result

The plan command succeeded and produced one build entry for `diag-gcc40-c-parse-boundary`, confirming the diagnostic target still evaluates after adding the function-after-include probes.

The runtime validation attempt was intentionally stopped after roughly 19 minutes because it was still rebuilding the Mes dependency from a fresh state directory. No `diag-gcc40-c-parse-boundary` derivation log had been emitted and target stdout/stderr were empty, so this slice does **not** claim a new GCC/TCC runtime boundary.

## Boundary

This commit scopes the next reduced boundary without changing the previously captured conclusion: `c_parse_init` placement alone was not the blocker, and the next useful runtime evidence should come from the new function-after-include matrix once run against a warm/non-corrupt validation state.
