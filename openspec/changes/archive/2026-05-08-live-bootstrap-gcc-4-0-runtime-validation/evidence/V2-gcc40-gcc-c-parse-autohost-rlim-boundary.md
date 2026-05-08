# GCC 4.0 c-parse auto-host rlim_t boundary after NEED64/gid/inline

Date: 2026-05-06

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

This extends the generated `auto-host.h` reduction after neutralizing the prior
reduced triggers:

```c
#define NEED_64BIT_HOST_WIDE_INT 1
#define gid_t int
#define inline
```

With those three triggers neutralized before including `system.h`, the next
prefix boundary is generated define 98:

```c
#define rlim_t long
```

Key probe results:

```text
autohost_defines_97_undef_need64_gid_inline_system rc=0
autohost_defines_98_undef_need64_gid_inline_system rc=139
autohost_define_98_only_system rc=0
autohost_defines_98_undef_need64_gid_inline_rlim_system rc=0
autohost_defines_99_undef_need64_gid_inline_system rc=139
autohost_defines_100_undef_need64_gid_inline_system rc=139
```

Unlike the earlier NEED64/gid/inline probes, define 98 alone did not reproduce
the crash. The reduced boundary is therefore the generated prefix context plus
`rlim_t` before `system.h`; undefining `rlim_t` alongside the three earlier
triggers makes prefix 98 pass.

Full config still segfaults; later generated typedef/config rewrites likely
remain after these four triggers are neutralized.
