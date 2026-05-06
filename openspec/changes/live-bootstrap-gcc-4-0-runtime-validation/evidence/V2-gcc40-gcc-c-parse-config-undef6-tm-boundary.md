# GCC 4.0 c-parse config undef6 and tm boundary

Date: 2026-05-06

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Active target: `bootstrap/gcc-4.0.ncl`

After neutralizing the six generated `auto-host.h` config triggers already
identified before `system.h`:

```c
#define NEED_64BIT_HOST_WIDE_INT 1
#define gid_t int
#define inline
#define rlim_t long
#define ssize_t int
#define uid_t int
```

the diagnostic proves the full generated `auto-host.h` + `system.h` probe no
longer segfaults. Appending the same undefs to a generated `config-undef6.h`
also lets `config + system.h` and `config + system.h + coretypes.h` compile.
The next reduced include boundary is `tm.h`.

Key diagnostic results:

```text
autohost_full_undef_need64_system rc=139
autohost_full_undef_need64_gid_inline_rlim_ssize_uid_system rc=0
cparse_config_undef6_system rc=0
cparse_undef6_inc_system rc=0
cparse_undef6_inc_coretypes rc=0
cparse_undef6_inc_tm rc=139
cparse_full_config_undef6 rc=139
```

`bootstrap/gcc-4.0.ncl` now appends those six undefs to generated
`build/gcc/config.h` after configure. A focused active build still reaches a
TinyCC segmentation fault at `gcc/c-parse.c`, consistent with the remaining
`tm.h`/post-config boundary rather than the earlier `auto-host.h + system.h`
trigger.
