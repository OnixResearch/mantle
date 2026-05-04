# musl-1.1.24-tcc bridge attempt evidence

Date: 2026-05-04

This evidence records a focused implementation investigation of `bootstrap/musl-1.1.24-tcc.ncl` after `tcc-musl-prep` started passing.

Exploratory changes tested but not retained because direct validation still failed:

- locate the unpacked musl `configure` script before running configure
- use a TinyCC-compatible x86_64 `__builtin_va_list` bridge in generated `bits/alltypes.h`
- reduce first-stage musl to static startup objects by dropping PIE/rcrt sources
- append `chmod u+rw $@` through `CC_CMD` so TinyCC-created objects are copyable
- replace `src/conf/sysconf.c` to avoid duplicate `_SC_PAGE_SIZE`/`_SC_PAGESIZE` switch cases that segfault bridge TinyCC
- replace `src/ctype/__ctype_get_mb_cur_max.c` with a first-stage constant implementation
- truncate x86_64 `src/internal/syscall.h` before legacy non-x86 syscall fixups, which also trigger bridge TinyCC preprocessor crashes
- remove the earlier `-D__DEFINED_struct_timespec` workaround because it suppresses `struct timespec` needed by `bits/stat.h`

Result: direct validation still exited `1`. The latest observed failure advanced past the original `sysconf.c`/object-permission boundary to `src/env/__init_tls.c` during `musl-1.1.24-tcc.drv` compilation. No implementation changes are committed in this slice.

Evidence files copied in this attempt:

- `V2-musl-1.1.24-tcc-bridge-attempt-validation-summary.json`
- `V2-musl-1.1.24-tcc-bridge-attempt-validation-summary.md`
- `V2-musl-1.1.24-tcc-bridge-attempt-build.stdout.log`
- `V2-musl-1.1.24-tcc-bridge-attempt-build.stderr.log`
- `V2-musl-1.1.24-tcc-bridge-attempt-root-derivation.log`
