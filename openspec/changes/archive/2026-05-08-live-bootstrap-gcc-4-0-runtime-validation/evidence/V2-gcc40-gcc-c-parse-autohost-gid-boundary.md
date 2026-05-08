# GCC 4.0 c-parse auto-host gid_t boundary after NEED_64BIT

## Summary

The previous diagnostic isolated the first generated `auto-host.h` crash trigger
to define 84, `NEED_64BIT_HOST_WIDE_INT`. This follow-up kept that macro
neutralized and reduced the next generated-config boundary.

With `NEED_64BIT_HOST_WIDE_INT` undefined before including `system.h`, the
sandbox-generated autohost prefix now passes through define 95 and crashes at
define 96:

```text
autohost_defines_95_undef_need64_system rc=0
autohost_defines_96_undef_need64_system rc=139
```

The generated define window identifies define 96 as:

```c
#define gid_t int
```

The single macro is sufficient for the reduced crash, and removing it alongside
`NEED_64BIT_HOST_WIDE_INT` clears the prefix-96 probe:

```text
autohost_define_96_only_system rc=139
autohost_defines_96_undef_need64_gid_system rc=0
```

The broader full-config path still crashes, so the active blocker remains a
sequence of generated config typedef/macro rewrites before `system.h`, not yet
parser-table or `yyparse` code.

## Evidence

Diagnostic scratch directory:

```text
.crunch-drain/gcc40-autohost-after-need64
```

Diagnostic derivation log:

```text
.crunch-drain/gcc40-autohost-after-need64/state/logs/xnrpkvnnpjdgfc5sahygnvb0zbvj88cv-diag-gcc40-c-parse-boundary.drv.log
```

Relevant matrix lines:

```text
diag-autohost-define: 084 #define NEED_64BIT_HOST_WIDE_INT 1
diag-autohost-define: 095 #define SIZEOF_VOID_P 0
diag-autohost-define: 096 #define gid_t int
diag-autohost-define: 097 #define inline
diag-cparse: compile autohost_defines_95_undef_need64_system
diag-cparse: autohost_defines_95_undef_need64_system rc=0
diag-cparse: compile autohost_defines_96_undef_need64_system
diag-cparse: autohost_defines_96_undef_need64_system rc=139
diag-cparse: compile autohost_define_96_only_system
diag-cparse: autohost_define_96_only_system rc=139
diag-cparse: compile autohost_defines_96_undef_need64_gid_system
diag-cparse: autohost_defines_96_undef_need64_gid_system rc=0
diag-cparse: compile autohost_defines_100_undef_need64_inline_system
diag-cparse: autohost_defines_100_undef_need64_inline_system rc=139
diag-cparse: compile autohost_defines_90_skip84_system
diag-cparse: autohost_defines_90_skip84_system rc=0
diag-cparse: compile autohost_defines_100_skip84_system
diag-cparse: autohost_defines_100_skip84_system rc=139
diag-cparse: compile autohost_full_undef_need64_system
diag-cparse: autohost_full_undef_need64_system rc=139
diag-cparse: compile cparse_ansidecl_system
diag-cparse: cparse_ansidecl_system rc=0
diag-cparse: compile cparse_inc_system_only
diag-cparse: cparse_inc_system_only rc=0
diag-cparse: cparse_inc_system rc=139
diag-cparse: cparse_full rc=139
```

Interpretation:

- `NEED_64BIT_HOST_WIDE_INT` is still the first autohost boundary.
- After neutralizing it, the next boundary is generated define 96,
  `#define gid_t int`.
- `gid_t` alone followed by `system.h` segfaults TinyCC, matching the earlier
  system-header class where config/compatibility macros perturb system typedefs.
- Prefix 96 with both `NEED_64BIT_HOST_WIDE_INT` and `gid_t` undefined passes.
- Full `auto-host.h` with only `NEED_64BIT_HOST_WIDE_INT` undefined still
  segfaults; later typedef rewrites such as `inline`, `rlim_t`, `ssize_t`, or
  `uid_t` may still need independent reduction.
