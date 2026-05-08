# GCC 4.0 c-parse system/prelude reduction

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

## What changed

Added a smaller diagnostic body helper and probes around the previously narrowed
`config-undef6.h + system.h + trivial function` failure. The new probes compare:

- declaration-only and global-only snippets under `config-undef6.h`;
- a bare function definition under `config-undef6.h` with no `system.h`;
- declaration/global/function variants after `system.h`;
- the existing progressive include-chain function matrix.

## Evidence

- Evidence directory: `evidence/gcc40-cparse-system-prelude-reduce-20260507/`
- Normalized result summary: `evidence/gcc40-cparse-system-prelude-reduce-20260507/probe-results.md`
- Machine-readable results: `evidence/gcc40-cparse-system-prelude-reduce-20260507/probe-results.json`
- Runner summary: `evidence/gcc40-cparse-system-prelude-reduce-20260507/validation-summary.md`
- Captured stdout transcript: `evidence/gcc40-cparse-system-prelude-reduce-20260507/build.stdout.log`

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-system-prelude-reduce-20260507 \
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
| `cparse_func_after_config_only` | `139` |
| `cparse_decl_after_system` | `0` |
| `cparse_global_after_system` | `0` |
| `cparse_oldstyle_func_after_system` | `139` |
| `cparse_int_func_after_system` | `139` |
| `cparse_func_after_system` | `139` |

This moves the boundary below `system.h`: a bare function definition after only
`config-undef6.h` returns `rc=139`, while declaration-only and global-only probes
pass with and without `system.h`.

## Interpretation

`system.h` and later GCC headers are not necessary to reproduce the C parser
segfault. The next high-ROI reduction is the function-definition parser shape
itself under `config-undef6.h`: split declaration specifiers, declarator,
parameter list, compound body, and empty-body handling before returning to full
`c-parse.c` normalization.
