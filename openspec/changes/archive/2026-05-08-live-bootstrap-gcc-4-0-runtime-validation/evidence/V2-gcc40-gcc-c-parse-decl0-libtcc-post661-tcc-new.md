# V2 gcc40 c-parse decl0 libtcc post661 prefix and tcc_new reduction

Task-ID: V2 focused TinyCC `libtcc.c` post-`tcc_compile` full-file reduction for the GCC 4.0 c-parse `decl0` predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Commands

Both replays run under a `/crunch/store` bind because the restored TinyCC and musl outputs embed logical Crunch store paths:

```sh
BASH=$(command -v bash)
nix shell nixpkgs#bubblewrap -c bwrap \
  --dev /dev --proc /proc --tmpfs /tmp --dir /crunch \
  --bind "$PWD/.crunch-drain/post621-restored-store" /crunch/store \
  --ro-bind /nix /nix --ro-bind /usr /usr \
  --ro-bind /run/current-system /run/current-system --ro-bind /home /home \
  --chdir "$PWD" "$BASH" \
  openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-post661-prefix-20260509/run-local-post661-prefix.sh \
  > openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-post661-prefix-20260509/focused-local-post661-prefix.txt 2>&1

nix shell nixpkgs#bubblewrap -c bwrap \
  --dev /dev --proc /proc --tmpfs /tmp --dir /crunch \
  --bind "$PWD/.crunch-drain/post621-restored-store" /crunch/store \
  --ro-bind /nix /nix --ro-bind /usr /usr \
  --ro-bind /run/current-system /run/current-system --ro-bind /home /home \
  --chdir "$PWD" "$BASH" \
  openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-tcc-new-20260509/run-local-tcc-new.sh \
  > openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-tcc-new-20260509/focused-local-tcc-new.txt 2>&1
```

Exit status: both 0.

## Evidence

- Prefix run directory: `evidence/gcc40-cparse-decl0-libtcc-post661-prefix-20260509/`
- Prefix script: `evidence/gcc40-cparse-decl0-libtcc-post661-prefix-20260509/run-local-post661-prefix.sh`
- Prefix log: `evidence/gcc40-cparse-decl0-libtcc-post661-prefix-20260509/focused-local-post661-prefix.txt`
- `tcc_compile_string` check directory: `evidence/gcc40-cparse-decl0-libtcc-compile-string-20260509/`
- `tcc_compile_string` script: `evidence/gcc40-cparse-decl0-libtcc-compile-string-20260509/run-local-compile-string.sh`
- `tcc_compile_string` log: `evidence/gcc40-cparse-decl0-libtcc-compile-string-20260509/focused-local-compile-string.txt`
- `tcc_new` run directory: `evidence/gcc40-cparse-decl0-libtcc-tcc-new-20260509/`
- `tcc_new` script: `evidence/gcc40-cparse-decl0-libtcc-tcc-new-20260509/run-local-tcc-new.sh`
- `tcc_new` log: `evidence/gcc40-cparse-decl0-libtcc-tcc-new-20260509/focused-local-tcc-new.txt`

## Prefix result

The replay preserves the known prerequisites:

- `native387_disabled` in `tccgen.c`
- `libtcc.c:tcc_compile` `CONFIG_TCC_ASM` island replaced with the active `tcc_error_noabort("asm not supported")` branch
- paired `va_start`/`va_end` cleanup from the line-621 prefix rail

The prefix script computes syntactically complete top-level closing-brace endpoints from the normalized source. This avoids counting malformed partial functions as valid `rc=139` evidence. A separate `tcc_compile_string` local reduction confirmed the complete `tcc_compile_string` function compiles under both flag sets; the apparent line-673 failure during early exploration was an incomplete-function artifact and is not used as evidence.

| Prefix | common | one_source |
| --- | ---: | ---: |
| through `tcc_compile`, line 662 | 0 | 0 |
| + `tcc_compile_string`, line 674 | 0 | 0 |
| + `tcc_define_symbol`, line 696 | 0 | 0 |
| + `tcc_undefine_symbol`, line 708 | 0 | 0 |
| + `tcc_cleanup`, line 723 | 0 | 0 |
| + `tcc_new`, line 894 | 139 | 139 |
| full asm-simplified `libtcc.c` | 139 | 139 |

## `tcc_new` reduction result

| Variant | common | one_source |
| --- | ---: | ---: |
| `base723` | 0 | 0 |
| `proto` | 0 | 0 |
| `empty` | 0 | 0 |
| `alloc` | 0 | 0 |
| `defaults` | 0 | 0 |
| `libpath` | 0 | 0 |
| `dummy_defines` | 0 | 0 |
| `version_decls` | 0 | 0 |
| `version_sscanf` | 139 | 139 |
| `version_sprintf_const` | 0 | 0 |
| `version_sprintf_expr` | 0 | 0 |
| `version_define_only` | 0 | 0 |
| full `tinyc_version` block | 139 | 139 |
| `standard_defines` | 0 | 0 |
| `target_x86` | 0 | 0 |
| `builtin_redirect` | 0 | 0 |
| full `tcc_new` | 139 | 139 |

## Interpretation

After the prior `tcc_compile` `CONFIG_TCC_ASM` island is simplified, the next syntactically valid full-file transition is adding `tcc_new`: prefixes through `tcc_cleanup` compile successfully, while the prefix through `tcc_new` crashes the predecessor TinyCC (`rc=139`) under both flag sets.

Within `tcc_new`, the reduced local trigger is the `sscanf(TCC_VERSION, "%d.%d.%d", &a, &b, &c);` statement in the `__TINYC__` version block. Declarations alone pass, `sprintf` with a constant passes, `sprintf` with the arithmetic expression passes, and `tcc_define_symbol` with the buffer passes. The combined original `tinyc_version` block fails because it includes the `sscanf` statement.

No direct `diag-tcc-decl0-runtime:*` markers executed; this remains predecessor-compile evidence only.
