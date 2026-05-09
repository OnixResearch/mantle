# GCC 4.0 c-parse decl0: libtcc post-894 copy_linker_arg boundary

Date: 2026-05-09

## Context

This continues the archived libtcc post-661/post-894 replay after these already-isolated predecessor compile triggers were bypassed:

- `tcc_compile()` `CONFIG_TCC_ASM` island
- `tcc_new()` `sscanf(TCC_VERSION, "%d.%d.%d", &a, &b, &c)`

The local replay uses the restored Crunch store at `.crunch-drain/post621-restored-store` and runs under bwrap with `/crunch/store` bound to that restored store.

## Evidence files

- `../gcc40-cparse-decl0-libtcc-post894-prefix-20260509/focused-local-post894-prefix.txt`
  - After bypassing the `tcc_new()` version `sscanf`, syntactically complete prefixes through line 1333 compile, while the full file still segfaults.
- `../gcc40-cparse-decl0-libtcc-set-linker-cumulative-20260509/focused-local-set-linker-cumulative.txt`
  - Cumulative reconstruction of `tcc_set_linker()` shows the first local crash when the `fini=` clause adds `copy_linker_arg(&s->fini_symbol, p, 0)`.
  - `cumulative_2_*` compiles; `cumulative_3_*` segfaults for both common and `-D ONE_SOURCE=1`.
- `../gcc40-cparse-decl0-libtcc-fini-copy-20260509/focused-local-fini-copy.txt`
  - Focused variants prove the crash is the `copy_linker_arg(&s->fini_symbol, p, 0)` call expression, not the `fini=` option branch itself:
    - ignore-only, `s->fini_symbol = NULL`, `s->fini_symbol = tcc_strdup(p)`, and `char **pp = &s->fini_symbol` all compile.
    - only the direct `copy_linker_arg(&s->fini_symbol, p, 0)` variant segfaults.
- `../gcc40-cparse-decl0-libtcc-set-linker-cumulative-copy-bypassed-20260509/focused-local-set-linker-cumulative-copy-bypassed.txt`
  - Replacing the `copy_linker_arg` field calls in the cumulative reconstruction lets all 0..14 cumulative `tcc_set_linker()` clauses compile for both flag sets.

## Boundary

The next confirmed libtcc predecessor compile trigger is the `copy_linker_arg(&s->fini_symbol, p, 0)` call in `tcc_set_linker()`.

This is narrower than a generic function-call or address-of-field problem: focused variants with the same branch shape, `&s->fini_symbol` address formation, and simpler assignment/call expressions compile. The failing shape is the specific helper call with the address-of-field first argument.

## Re-run

```sh
BASH=$(command -v bash)
nix shell nixpkgs#bubblewrap -c bwrap \
  --dev /dev --proc /proc --tmpfs /tmp --dir /crunch \
  --bind "$PWD/.crunch-drain/post621-restored-store" /crunch/store \
  --ro-bind /nix /nix --ro-bind /usr /usr \
  --ro-bind /run/current-system /run/current-system \
  --ro-bind /home /home --chdir "$PWD" "$BASH" \
  openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-fini-copy-20260509/run-local-fini-copy.sh
```
