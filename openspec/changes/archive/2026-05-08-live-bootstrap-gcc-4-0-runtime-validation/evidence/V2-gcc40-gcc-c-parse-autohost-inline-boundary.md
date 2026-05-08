# GCC 4.0 c-parse auto-host inline boundary after NEED64/gid

Date: 2026-05-06

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`

This extends the generated `auto-host.h` reduction after the previous reduced
triggers:

```c
#define NEED_64BIT_HOST_WIDE_INT 1
#define gid_t int
```

With both known triggers neutralized before including `system.h`, the next
single generated define boundary is define 97:

```c
#define inline
```

Key probe results:

```text
autohost_defines_96_undef_need64_gid_system rc=0
autohost_defines_97_undef_need64_gid_system rc=139
autohost_define_97_only_system rc=139
autohost_defines_97_undef_need64_gid_inline_system rc=0
autohost_defines_98_undef_need64_gid_system rc=139
autohost_defines_99_undef_need64_gid_system rc=139
autohost_defines_100_undef_need64_gid_system rc=139
```

The result matches the earlier `ansidecl.h` inline boundary: an empty `inline`
macro followed by GCC 4.0 `system.h`/`sys/types.h` is sufficient to make TinyCC
segfault under the c-parse compile flags. Undefining `inline` together with the
two earlier generated triggers lets the first 97 generated defines pass.

Full config still segfaults; later generated typedef/config rewrites likely
remain after these three triggers are neutralized.
