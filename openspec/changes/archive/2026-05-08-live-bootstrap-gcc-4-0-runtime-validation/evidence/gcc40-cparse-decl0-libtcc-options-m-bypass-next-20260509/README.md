# GCC 4.0 c-parse decl0: libtcc options_m bypass and tcc_parse_args switch boundary

Date: 2026-05-09

## Context

This continues the restored-output local replay after bypassing earlier local predecessor compile triggers:

- `tcc_compile()` `CONFIG_TCC_ASM` island
- `tcc_new()` `sscanf(TCC_VERSION, "%d.%d.%d", &a, &b, &c)`
- `tcc_set_linker()` direct `copy_linker_arg(&s->...)` field-call sites
- `options_m[]` `#ifdef TCC_TARGET_X86_64` island inside the static initializer

The replay uses `.crunch-drain/post621-restored-store` bound as `/crunch/store` inside bwrap. Evidence remains focused predecessor-compile replay against restored outputs, not direct Crunch runtime-marker proof.

## Evidence files

- `focused-local-options-m-bypass-next.txt`
  - Normalizes/removes one `options_m` target-gated preprocessor island and confirms the normalized full file still segfaults.
  - Generated top-level endpoints show complete prefixes through `args_parser_listfile()` compile under both common flags and `-D ONE_SOURCE=1`; full `tcc_parse_args` remains failing.
- `focused-local-post-options-m-functions.txt`
  - Cleaner syntactically complete endpoint proof after the `options_m` bypass.
  - `through_options_m`, `parse_option_D`, `args_parser_add_file`, `args_parser_make_argv`, and `args_parser_listfile` all compile with `rc=0` under both flag sets.
- `focused-local-tcc-parse-args-signature.txt`
  - `tcc_parse_args` prototype, empty body, no-`PUB_FUNC` body, alternate name, and declaration-only body variants compile.
  - `static int tcc_parse_args(...)` is a separate conflicting-shape crash and is not the upstream boundary.
- `focused-local-tcc-parse-args-body.txt`
  - Body variants through declarations, `cstr_new`, a minimal while loop, file branch, and option lookup loop compile.
  - Full `tcc_parse_args` body still segfaults.
- `focused-local-tcc-parse-args-cases.txt`
  - Adding the cumulative `switch(popt->index)` body beginning with the help/bench cases segfaults under both flag sets.

## Boundary

After normalizing the `options_m[]` preprocessor island, the next local predecessor compile trigger is inside `tcc_parse_args()`: the function signature and early option-lookup shell compile, but adding the real `switch(popt->index)` case body crashes the predecessor compiler.

Key lines:

```text
diag-libtcc-post-options-m-functions: compile post_options_m_fn_04_args_parser_listfile.c_common ... rc=0
diag-libtcc-post-options-m: compile tcc_parse_args_body_04_lookup_loop.c_common ... rc=0
diag-libtcc-post-options-m: compile tcc_parse_args_body_05_full_exact_body.c_common ... rc=-11
diag-libtcc-post-options-m: compile tcc_parse_args_cases_00_help_bench.c_common ... rc=-11
diag-libtcc-post-options-m: compile tcc_parse_args_cases_00_help_bench.c_D_ONE_SOURCE_1 ... rc=-11
```

## Re-run

```sh
BASH=$(command -v bash)
nix shell nixpkgs#bubblewrap -c bwrap   --dev /dev --proc /proc --tmpfs /tmp --dir /crunch   --bind "$PWD/.crunch-drain/post621-restored-store" /crunch/store   --ro-bind /nix /nix --ro-bind /usr /usr   --ro-bind /run/current-system /run/current-system   --ro-bind /home /home --chdir "$PWD" "$BASH"   openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-options-m-bypass-next-20260509/run-local-post-options-m-functions.sh
```
