# V2 GCC 4.0 c-parse system chunk boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation

## Summary

A focused diagnostic build of `bootstrap/diag-gcc40-c-parse-boundary.ncl` further narrowed the TinyCC c-parse include boundary. The previous slice showed that `system.h` alone compiles, but `system.h` after GCC config/header context segfaults. This slice reduces that interaction to a small valid include sequence: `ansidecl.h` followed by the early `system.h` include group through `<sys/types.h>`.

The diagnostic result remains an expected build failure (`rc=1`) because the target intentionally reproduces the final `gcc/c-parse.c` TinyCC segmentation fault after the matrix. The reduction matrix gives the material boundary:

- baseline chunks without prior `ansidecl.h` compile:
  - `chunk_safe_ctype rc=0`
  - `chunk_sys_types rc=0`
  - `chunk_errno rc=0`
- prior `ansidecl.h` plus early `system.h` chunks compile until `safe-ctype.h`:
  - `ansidecl_chunk_stdarg rc=0`
  - `ansidecl_chunk_va_copy rc=0`
  - `ansidecl_chunk_stddef rc=0`
  - `ansidecl_chunk_stdio rc=0`
  - `ansidecl_chunk_stdio_macros rc=0`
  - `ansidecl_chunk_safe_ctype rc=0`
- adding `<sys/types.h>` after `ansidecl.h` and the preceding early `system.h` include group segfaults TinyCC:
  - `ansidecl_chunk_sys_types rc=139`
  - `ansidecl_chunk_errno rc=139`
  - `ansidecl_chunk_string rc=139`

This makes the next boundary narrower than the whole generated `config.h`/`system.h` pair: it is the TinyCC frontend/preprocessor state produced by `ansidecl.h` plus the early `system.h` sequence through `<safe-ctype.h>`, with the first reproduced crash at the following `<sys/types.h>` include.

## Command

```sh
rm -rf .crunch-drain/gcc40-system-chunks-final && mkdir -p .crunch-drain/gcc40-system-chunks-final/store
export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/dk9qhjgg469lv6mriys7v4c59igarmvx-bubblewrap-0.11.1/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig"
export CC=clang
export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
export CARGO_TARGET_DIR=/tmp/crunch-target-gcc40-cparse
cargo run -- build --store "$PWD/.crunch-drain/gcc40-system-chunks-final/store" bootstrap/diag-gcc40-c-parse-boundary.ncl > .crunch-drain/gcc40-system-chunks-final/build.log 2>&1
```

Exit status: `1` (expected diagnostic failure after recording the matrix).

Focused transcript: `V2-gcc40-gcc-c-parse-system-chunk-boundary-build.diag.log`.

## Boundary excerpt

```text
diag-cparse: compile cparse_empty
diag-cparse: cparse_empty rc=0
diag-cparse: compile cparse_inc_ansidecl
diag-cparse: cparse_inc_ansidecl rc=0
diag-cparse: compile cparse_inc_autohost
diag-cparse: cparse_inc_autohost rc=0
diag-cparse: compile cparse_manual_config
diag-cparse: cparse_manual_config rc=0
diag-cparse: compile cparse_manual_config_nospace
diag-cparse: cparse_manual_config_nospace rc=0
diag-cparse: compile cparse_inc_config
diag-cparse: cparse_inc_config rc=0
diag-cparse: compile cparse_manual_config_system
diag-cparse: cparse_manual_config_system rc=139
diag-cparse: compile cparse_autohost_system
diag-cparse: cparse_autohost_system rc=139
diag-cparse: compile cparse_ansidecl_system
diag-cparse: cparse_ansidecl_system rc=139
diag-cparse: compile cparse_inc_system_only
diag-cparse: cparse_inc_system_only rc=0
diag-cparse: compile chunk_safe_ctype
diag-cparse: chunk_safe_ctype rc=0
diag-cparse: compile chunk_sys_types
diag-cparse: chunk_sys_types rc=0
diag-cparse: compile chunk_errno
diag-cparse: chunk_errno rc=0
diag-cparse: compile ansidecl_chunk_stdarg
diag-cparse: ansidecl_chunk_stdarg rc=0
diag-cparse: compile ansidecl_chunk_va_copy
diag-cparse: ansidecl_chunk_va_copy rc=0
diag-cparse: compile ansidecl_chunk_stddef
diag-cparse: ansidecl_chunk_stddef rc=0
diag-cparse: compile ansidecl_chunk_stdio
diag-cparse: ansidecl_chunk_stdio rc=0
diag-cparse: compile ansidecl_chunk_stdio_macros
diag-cparse: ansidecl_chunk_stdio_macros rc=0
diag-cparse: compile ansidecl_chunk_safe_ctype
diag-cparse: ansidecl_chunk_safe_ctype rc=0
diag-cparse: compile ansidecl_chunk_sys_types
diag-cparse: ansidecl_chunk_sys_types rc=139
diag-cparse: compile ansidecl_chunk_errno
diag-cparse: ansidecl_chunk_errno rc=139
diag-cparse: compile ansidecl_chunk_string
diag-cparse: ansidecl_chunk_string rc=139
diag-cparse: compile cparse_inc_system
diag-cparse: cparse_inc_system rc=139
diag-cparse: compile cparse_inc_coretypes
diag-cparse: cparse_inc_coretypes rc=139
diag-cparse: compile cparse_inc_tm
diag-cparse: cparse_inc_tm rc=139
diag-cparse: compile cparse_inc_tree
diag-cparse: cparse_inc_tree rc=139
diag-cparse: compile cparse_inc_langhooks
diag-cparse: cparse_inc_langhooks rc=139
diag-cparse: compile cparse_inc_cpplib
diag-cparse: cparse_inc_cpplib rc=139
diag-cparse: compile cparse_inc_ctree
diag-cparse: cparse_inc_ctree rc=139
diag-cparse: compile cparse_header_only
diag-cparse: cparse_header_only rc=139
diag-cparse: compile cparse_after_yytranslate
diag-cparse: cparse_after_yytranslate rc=139
diag-cparse: compile cparse_after_yyr1
diag-cparse: cparse_after_yyr1 rc=139
diag-cparse: compile cparse_after_yyr2
diag-cparse: cparse_after_yyr2 rc=139
diag-cparse: compile cparse_before_yytable
diag-cparse: cparse_before_yytable rc=139
diag-cparse: compile cparse_before_yyparse
diag-cparse: cparse_before_yyparse rc=139
diag-cparse: compile cparse_full
diag-cparse: cparse_full rc=139

```
