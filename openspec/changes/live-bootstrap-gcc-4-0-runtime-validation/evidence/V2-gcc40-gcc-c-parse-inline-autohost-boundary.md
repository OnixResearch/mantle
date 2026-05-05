# V2 GCC 4.0 c-parse inline/autohost boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation

## Summary

A focused diagnostic build of `bootstrap/diag-gcc40-c-parse-boundary.ncl` isolated and partially repaired the previous `ansidecl.h` + `<sys/types.h>` TinyCC crash. GCC 4.0's `ansidecl.h` rewrote `inline` to an empty macro when TinyCC reports an old GCC-compatible version. A minimal probe shows that `#define inline` followed by `<sys/types.h>` deterministically segfaults TinyCC (`rc=139`), while keeping `inline` as `__inline__` compiles (`rc=0`).

The diagnostic now patches GCC 4.0 `include/ansidecl.h` to keep `inline` as `__inline__` for this bootstrap handoff. With that source normalization:

- `ansidecl.h` + `<sys/types.h>` now compiles (`ansidecl_chunk_sys_types_only rc=0`).
- the broader `ansidecl.h` + early `system.h` chunks now compile through string headers (`ansidecl_chunk_string rc=0`).
- `ansidecl.h` + `system.h` now compiles (`cparse_ansidecl_system rc=0`).

The remaining `config.h` / `auto-host.h` path still crashes. A prefix reduction over generated `auto-host.h` definitions shows:

- `autohost_defines_10_system rc=0`
- `autohost_defines_25_system rc=0`
- `autohost_defines_50_system rc=0`
- `autohost_defines_100_system rc=139`
- `autohost_defines_200_system rc=139`
- `autohost_defines_400_system rc=139`

The full c-parse diagnostic remains an expected build failure (`rc=1`) because the target intentionally reproduces the remaining TinyCC `gcc/c-parse.c` segmentation fault after the matrix.

## Command

```sh
rm -rf .crunch-drain/gcc40-autohost-reduce && mkdir -p .crunch-drain/gcc40-autohost-reduce/store
export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/dk9qhjgg469lv6mriys7v4c59igarmvx-bubblewrap-0.11.1/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig"
export CC=clang
export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
export CARGO_TARGET_DIR=/tmp/crunch-target-gcc40-cparse
cargo run -- build --store "$PWD/.crunch-drain/gcc40-autohost-reduce/store" bootstrap/diag-gcc40-c-parse-boundary.ncl > .crunch-drain/gcc40-autohost-reduce/build.log 2>&1
```

Exit status: `1` (expected diagnostic failure after recording the matrix).

Focused transcript: `V2-gcc40-gcc-c-parse-inline-autohost-boundary-build.diag.log`.

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
diag-cparse: compile autohost_defines_10_system
diag-cparse: autohost_defines_10_system rc=0
diag-cparse: compile autohost_defines_25_system
diag-cparse: autohost_defines_25_system rc=0
diag-cparse: compile autohost_defines_50_system
diag-cparse: autohost_defines_50_system rc=0
diag-cparse: compile autohost_defines_100_system
diag-cparse: autohost_defines_100_system rc=139
diag-cparse: compile autohost_defines_200_system
diag-cparse: autohost_defines_200_system rc=139
diag-cparse: compile autohost_defines_400_system
diag-cparse: autohost_defines_400_system rc=139
diag-cparse: compile cparse_manual_config_system
diag-cparse: cparse_manual_config_system rc=139
diag-cparse: compile cparse_autohost_system
diag-cparse: cparse_autohost_system rc=139
diag-cparse: compile cparse_ansidecl_system
diag-cparse: cparse_ansidecl_system rc=0
diag-cparse: compile cparse_inc_system_only
diag-cparse: cparse_inc_system_only rc=0
diag-cparse: compile chunk_safe_ctype
diag-cparse: chunk_safe_ctype rc=0
diag-cparse: compile chunk_sys_types
diag-cparse: chunk_sys_types rc=0
diag-cparse: compile chunk_errno
diag-cparse: chunk_errno rc=0
diag-cparse: compile ansimacro_gcc_version_sys_types
diag-cparse: ansimacro_gcc_version_sys_types rc=0
diag-cparse: compile ansimacro_ansi_defs_sys_types
diag-cparse: ansimacro_ansi_defs_sys_types rc=0
diag-cparse: compile ansimacro_undef_keywords_sys_types
diag-cparse: ansimacro_undef_keywords_sys_types rc=139
diag-cparse: compile ansimacro_undef_const_sys_types
diag-cparse: ansimacro_undef_const_sys_types rc=0
diag-cparse: compile ansimacro_undef_volatile_sys_types
diag-cparse: ansimacro_undef_volatile_sys_types rc=0
diag-cparse: compile ansimacro_undef_signed_sys_types
diag-cparse: ansimacro_undef_signed_sys_types rc=0
diag-cparse: compile ansimacro_undef_inline_sys_types
diag-cparse: ansimacro_undef_inline_sys_types rc=0
diag-cparse: compile ansimacro_inline_empty_sys_types
diag-cparse: ansimacro_inline_empty_sys_types rc=139
diag-cparse: compile ansimacro_inline_builtin_sys_types
diag-cparse: ansimacro_inline_builtin_sys_types rc=0
diag-cparse: compile ansimacro_attribute_blank_sys_types
diag-cparse: ansimacro_attribute_blank_sys_types rc=0
diag-cparse: compile ansimacro_extension_blank_sys_types
diag-cparse: ansimacro_extension_blank_sys_types rc=0
diag-cparse: compile ansidecl_chunk_sys_types_only
diag-cparse: ansidecl_chunk_sys_types_only rc=0
diag-cparse: compile ansidecl_chunk_stdarg
diag-cparse: ansidecl_chunk_stdarg rc=0
diag-cparse: compile ansidecl_chunk_stdarg_sys_types
diag-cparse: ansidecl_chunk_stdarg_sys_types rc=0
diag-cparse: compile ansidecl_chunk_va_copy
diag-cparse: ansidecl_chunk_va_copy rc=0
diag-cparse: compile ansidecl_chunk_va_copy_sys_types
diag-cparse: ansidecl_chunk_va_copy_sys_types rc=0
diag-cparse: compile ansidecl_chunk_stddef
diag-cparse: ansidecl_chunk_stddef rc=0
diag-cparse: compile ansidecl_chunk_stddef_sys_types
diag-cparse: ansidecl_chunk_stddef_sys_types rc=0
diag-cparse: compile ansidecl_chunk_stdio
diag-cparse: ansidecl_chunk_stdio rc=0
diag-cparse: compile ansidecl_chunk_stdio_sys_types
diag-cparse: ansidecl_chunk_stdio_sys_types rc=0
diag-cparse: compile ansidecl_chunk_stdio_macros
diag-cparse: ansidecl_chunk_stdio_macros rc=0
diag-cparse: compile ansidecl_chunk_stdio_macros_sys_types
diag-cparse: ansidecl_chunk_stdio_macros_sys_types rc=0
diag-cparse: compile ansidecl_chunk_safe_ctype
diag-cparse: ansidecl_chunk_safe_ctype rc=0
diag-cparse: compile ansidecl_chunk_safe_ctype_sys_types
diag-cparse: ansidecl_chunk_safe_ctype_sys_types rc=0
diag-cparse: compile ansidecl_chunk_sys_types
diag-cparse: ansidecl_chunk_sys_types rc=0
diag-cparse: compile ansidecl_chunk_errno
diag-cparse: ansidecl_chunk_errno rc=0
diag-cparse: compile ansidecl_chunk_string
diag-cparse: ansidecl_chunk_string rc=0
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
