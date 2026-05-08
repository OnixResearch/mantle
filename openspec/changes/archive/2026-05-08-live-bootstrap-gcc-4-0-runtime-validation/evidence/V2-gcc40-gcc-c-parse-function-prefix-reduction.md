# GCC 4.0 c-parse function-prefix reduction

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

## What changed

Added a smaller prefix matrix around the previously narrowed function-definition
crash. The slice also removed the already-proven long `cparse_func_after_*` include
chain from the active diagnostic so the generated builder stays below the OS
argument-size ceiling; the committed matrix evidence already preserves those include
results.

The new probes compare:

- plain source with no `config-undef6.h`;
- exact config-only function body with no trailing marker;
- config-only function declarator/open-brace/local-decl prefixes;
- existing config-only declaration/global/function controls.

## Evidence

- Evidence directory: `evidence/gcc40-cparse-function-prefix-reduce-20260507/`
- Normalized result summary: `evidence/gcc40-cparse-function-prefix-reduce-20260507/probe-results.md`
- Machine-readable results: `evidence/gcc40-cparse-function-prefix-reduce-20260507/probe-results.json`
- Runner summary: `evidence/gcc40-cparse-function-prefix-reduce-20260507/validation-summary.md`
- Captured stdout transcript: `evidence/gcc40-cparse-function-prefix-reduce-20260507/build.stdout.log`

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-function-prefix-reduce-20260507 \
  --resume
```

Exit status: `1` (`build-failed`; expected for this diagnostic because the
probes intentionally preserve rc=139 failures).

## Result

Key probe results:

| Probe | rc |
|---|---:|
| `cparse_plain_decl_no_config` | `0` |
| `cparse_plain_global_no_config` | `0` |
| `cparse_plain_func_no_config` | `139` |
| `cparse_func_exact_config_only` | `139` |
| `cparse_func_prefix_declarator_config_only` | `139` |
| `cparse_func_prefix_open_brace_config_only` | `139` |
| `cparse_func_prefix_local_decl_config_only` | `139` |
| `cparse_decl_after_config_only` | `0` |
| `cparse_global_after_config_only` | `0` |
| `cparse_func_after_config_only` | `139` |

## Interpretation

`config-undef6.h` is not required for the minimal crash: no-config declarations
and globals pass, but a no-config function definition returns `139`. The crash
also appears before a compound body is necessary: the config-only function
declarator prefix without a semicolon returns `139`.

The next reduction should compare function declarator parsing against declaration
termination and nonfunction declarator prefixes, rather than adding more complete
function bodies.
