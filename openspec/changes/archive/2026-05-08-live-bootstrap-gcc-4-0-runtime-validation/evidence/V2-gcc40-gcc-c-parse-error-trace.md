# V2: GCC 4.0 c-parse declaration error trace

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured
Date: 2026-05-07
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Evidence directory: `evidence/gcc40-cparse-error-trace-20260507/`

## Scope

Added focused trace probes beside the existing declaration/declarator matrix. The probes print the exact source line for representative valid, EOF-incomplete, and malformed declarations, then try driver phase splits (`-S`, `-E`, and preprocessed compile) before the already-recorded direct `-c` matrix.

## Result

The runtime reached `diag-gcc40-c-parse-boundary` and emitted per-probe trace output.

Key trace findings:

```text
# GCC 4.0 c-parse declaration error trace results

- `cparse_trace_valid_var_semicolon`
  - source: `int cparse_trace_valid_var_probe;`
  - assembly_rc: `60`
  - preprocess_rc: `139`
  - preprocessed_lines: `0`
  - preprocessed_compile_rc: `skipped`
  - assembly_stderr: `tcc: error: invalid option -- 'invalid option -- '%s'' | tcc: error: invalid option -- 'invalid option -- '%s''`
  - preprocess_stderr: `Segmentation fault (core dumped)`
- `cparse_trace_eof_var_prefix`
  - source: `int cparse_trace_eof_var_probe`
  - assembly_rc: `60`
  - preprocess_rc: `139`
  - preprocessed_lines: `0`
  - preprocessed_compile_rc: `skipped`
  - assembly_stderr: `tcc: error: invalid option -- 'invalid option -- '%s'' | tcc: error: invalid option -- 'invalid option -- '%s''`
  - preprocess_stderr: `Segmentation fault (core dumped)`
- `cparse_trace_bad_init_semicolon`
  - source: `int cparse_trace_bad_init_probe = ;`
  - assembly_rc: `60`
  - preprocess_rc: `139`
  - preprocessed_lines: `0`
  - preprocessed_compile_rc: `skipped`
  - assembly_stderr: `tcc: error: invalid option -- 'invalid option -- '%s'' | tcc: error: invalid option -- 'invalid option -- '%s''`
  - preprocess_stderr: `Segmentation fault (core dumped)`
- `cparse_trace_bad_comma_semicolon`
  - source: `int cparse_trace_bad_comma_probe, ;`
  - assembly_rc: `60`
  - preprocess_rc: `139`
  - preprocessed_lines: `0`
  - preprocessed_compile_rc: `skipped`
  - assembly_stderr: `tcc: error: invalid option -- 'invalid option -- '%s'' | tcc: error: invalid option -- 'invalid option -- '%s''`
  - preprocess_stderr: `Segmentation fault (core dumped)`

## Direct rc subset
- `cparse_plain_func_decl_semicolon_exact_no_config` rc=`0`
- `cparse_plain_var_prefix_no_config` rc=`139`
- `cparse_plain_pointer_prefix_no_config` rc=`139`
- `cparse_plain_var_semicolon_exact_no_config` rc=`0`
- `cparse_plain_pointer_semicolon_exact_no_config` rc=`0`
- `cparse_plain_bad_init_semicolon_no_config` rc=`139`
- `cparse_plain_bad_param_semicolon_no_config` rc=`139`
- `cparse_plain_bad_comma_semicolon_no_config` rc=`139`
- `cparse_func_decl_semicolon_config_only` rc=`0`
- `cparse_var_prefix_config_only` rc=`139`
- `cparse_pointer_prefix_config_only` rc=`139`
- `cparse_var_semicolon_config_only` rc=`0`
- `cparse_pointer_semicolon_config_only` rc=`0`
- `cparse_bad_init_semicolon_config_only` rc=`139`
- `cparse_bad_param_semicolon_config_only` rc=`139`
- `cparse_bad_comma_semicolon_config_only` rc=`139`
```

Interpretation:

- The direct `-c` matrix remains the authoritative declaration parser boundary: valid semicolon declarations return `rc=0`, while EOF-incomplete and malformed declaration/error paths return `rc=139`.
- `gcc40-cc -S` is not a usable phase splitter for this TinyCC handoff: even the valid declaration returns `rc=60` with corrupted literal `%s` invalid-option diagnostics.
- `gcc40-cc -E` is also not a usable phase splitter here: even the valid declaration returns `rc=139` before producing preprocessed output.
- Therefore flag-only phase tracing cannot isolate the declaration parser action further. The next useful instrumentation must be source-level TinyCC parser/error-reporting instrumentation or a dedicated diagnostic compiler, not more GCC source-shape probes or `-E`/`-S` splits.

## Verification

- command: `timeout 600 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json --store .crunch-drain/gcc40-cparse-error-trace-20260507-store --state-dir .crunch-drain/gcc40-runtime-v4-20260507-state bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence --resume`
- result: fail expected (`build_exit_code=1`) because the diagnostic target intentionally reaches crashing parser probes.
- transcript: `evidence/gcc40-cparse-error-trace-20260507/build.stdout.log`
- normalized trace: `evidence/gcc40-cparse-error-trace-20260507/trace-results.md`
