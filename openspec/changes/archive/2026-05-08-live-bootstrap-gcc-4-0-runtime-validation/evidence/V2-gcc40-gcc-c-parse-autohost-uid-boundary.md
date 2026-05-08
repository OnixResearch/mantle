# GCC 4.0 c-parse auto-host uid_t boundary after NEED64/gid/inline/rlim/ssize

Date: 2026-05-06

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

This extends the generated `auto-host.h` reduction after neutralizing the prior
reduced triggers:

```c
#define NEED_64BIT_HOST_WIDE_INT 1
#define gid_t int
#define inline
#define rlim_t long
#define ssize_t int
```

With those five triggers neutralized before including `system.h`, the next
prefix boundary is generated define 100:

```c
#define uid_t int
```

Key probe results:

```text
autohost_defines_99_undef_need64_gid_inline_rlim_ssize_system rc=0
autohost_defines_100_undef_need64_gid_inline_rlim_ssize_system rc=139
autohost_define_100_only_system rc=139
autohost_defines_100_undef_need64_gid_inline_rlim_ssize_uid_system rc=0
```

Generated define 100 is independently sufficient in the reduced probe, and
undefining it together with the previous five triggers makes prefix 100 pass.
Full config still segfaults; later generated config rewrites remain reducer
targets.
