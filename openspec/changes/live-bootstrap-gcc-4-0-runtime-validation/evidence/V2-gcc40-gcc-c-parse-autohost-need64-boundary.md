# GCC 4.0 c-parse auto-host NEED_64BIT boundary

## Summary

Focused diagnostic validation identified the 84th generated `auto-host.h`
`#define` as:

```c
#define NEED_64BIT_HOST_WIDE_INT 1
```

Under the real `gcc/c-parse.c` diagnostic compile flags, this single macro is
sufficient to trigger the TinyCC segmentation fault when followed by
`system.h`:

```text
autohost_defines_83_system rc=0
autohost_defines_84_system rc=139
autohost_defines_84_skip84_system rc=0
autohost_defines_85_skip84_system rc=0
autohost_define_84_only_system rc=139
autohost_defines_84_undef_need64_system rc=0
```

The full `auto-host.h` plus an immediate `#undef NEED_64BIT_HOST_WIDE_INT`
still segfaults, so there are later config macros that also need reduction;
this slice only proves the first deterministic autohost macro boundary.

## Evidence

Diagnostic run scratch directory:

```text
.crunch-drain/gcc40-autohost-need64
```

Diagnostic derivation log:

```text
.crunch-drain/gcc40-autohost-need64/state/logs/lrnkw1xhx7m1msg8p3cygjafb2s229gq-diag-gcc40-c-parse-boundary.drv.log
```

Relevant matrix lines:

```text
diag-autohost-define: 080 #define HAVE_WCHAR_H 1
diag-autohost-define: 081 #define HAVE_WORKING_FORK 1
diag-autohost-define: 082 #define HAVE_WORKING_VFORK 1
diag-autohost-define: 083 #define MKDIR_TAKES_ONE_ARG 1
diag-autohost-define: 084 #define NEED_64BIT_HOST_WIDE_INT 1
diag-autohost-define: 085 #define PACKAGE "gcc"
diag-autohost-define: 086 #define PACKAGE_BUGREPORT ""
diag-cparse: compile autohost_defines_83_system
diag-cparse: autohost_defines_83_system rc=0
diag-cparse: compile autohost_defines_84_system
diag-cparse: autohost_defines_84_system rc=139
diag-cparse: compile autohost_defines_84_skip84_system
diag-cparse: autohost_defines_84_skip84_system rc=0
diag-cparse: compile autohost_defines_85_skip84_system
diag-cparse: autohost_defines_85_skip84_system rc=0
diag-cparse: compile autohost_define_84_only_system
diag-cparse: autohost_define_84_only_system rc=139
diag-cparse: compile autohost_defines_84_undef_need64_system
diag-cparse: autohost_defines_84_undef_need64_system rc=0
diag-cparse: compile autohost_full_undef_need64_system
diag-cparse: autohost_full_undef_need64_system rc=139
diag-cparse: compile autohost_defines_80_84_system
diag-cparse: autohost_defines_80_84_system rc=139
diag-cparse: compile cparse_ansidecl_system
diag-cparse: cparse_ansidecl_system rc=0
diag-cparse: compile cparse_inc_system_only
diag-cparse: cparse_inc_system_only rc=0
diag-cparse: cparse_inc_system rc=139
diag-cparse: cparse_full rc=139
```

Interpretation:

- The 80..86 generated define window shows define 84 is
  `NEED_64BIT_HOST_WIDE_INT`.
- Prefix 83 passes and prefix 84 segfaults.
- Prefix 84 with define 84 skipped passes.
- Prefix 85 with define 84 skipped also passes, so the immediately following
  `PACKAGE` define is not the cause.
- Define 84 alone followed by `system.h` segfaults, proving the macro is
  independently sufficient for this reduced TinyCC crash.
- Prefix 84 with `#undef NEED_64BIT_HOST_WIDE_INT` before `system.h` passes,
  confirming that the macro binding is necessary for this first boundary.
- Full `auto-host.h` with `#undef NEED_64BIT_HOST_WIDE_INT` still segfaults,
  preserving the next reducer target after this first macro boundary.
