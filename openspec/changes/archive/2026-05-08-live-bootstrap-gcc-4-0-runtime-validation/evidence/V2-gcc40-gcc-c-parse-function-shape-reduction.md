# GCC 4.0 c-parse function-shape reduction

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

## What changed

Added config-only function-definition shape probes to separate a single spelling
of `void f(void) { }` from the C parser's broader function-definition path.
The new probes compare declaration/global controls with old-style, int-return,
empty-statement body, local-declaration body, static, extern, and original void
function definitions under only `config-undef6.h`.

## Evidence

- Evidence directory: `evidence/gcc40-cparse-function-shape-reduce-20260507/`
- Normalized result summary: `evidence/gcc40-cparse-function-shape-reduce-20260507/probe-results.md`
- Machine-readable results: `evidence/gcc40-cparse-function-shape-reduce-20260507/probe-results.json`
- Runner summary: `evidence/gcc40-cparse-function-shape-reduce-20260507/validation-summary.md`
- Captured stdout transcript: `evidence/gcc40-cparse-function-shape-reduce-20260507/build.stdout.log`

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-function-shape-reduce-20260507 \
  --resume
```

Exit status: `1` (`build-failed`; expected for this diagnostic because the
probes intentionally preserve rc=139 failures).

## Result

Key probe results:

| Probe | rc |
|---|---:|
| `cparse_decl_after_config_only` | `0` |
| `cparse_global_after_config_only` | `0` |
| `cparse_oldstyle_func_after_config_only` | `139` |
| `cparse_int_func_after_config_only` | `139` |
| `cparse_func_empty_stmt_after_config_only` | `139` |
| `cparse_func_local_decl_after_config_only` | `139` |
| `cparse_static_func_after_config_only` | `139` |
| `cparse_extern_func_after_config_only` | `139` |
| `cparse_func_after_config_only` | `139` |

## Interpretation

The crash is not tied to `system.h`, later GCC headers, return type, parameter
style, storage class, or empty-body spelling. It reproduces for every valid
whole-function definition tested under `config-undef6.h`, while declarations and
globals pass.

The next reduction should isolate the transition from declaration parsing into
function-definition parsing: compound body/open brace and function finalization
paths under `config-undef6.h`.
