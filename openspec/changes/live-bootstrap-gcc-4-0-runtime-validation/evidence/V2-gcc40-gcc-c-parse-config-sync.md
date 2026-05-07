# GCC 4.0 c-parse diagnostic config sync

Date: 2026-05-06

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Active target: `bootstrap/gcc-4.0.ncl`

## Change under test

The focused diagnostic now aligns its generated `gcc/config.h` with the active
`bootstrap/gcc-4.0.ncl` handoff by appending the same six auto-host cleanup
undefs immediately after `tree-check.h config.h` generation:

```c
#undef NEED_64BIT_HOST_WIDE_INT
#undef gid_t
#undef inline
#undef rlim_t
#undef ssize_t
#undef uid_t
```

The previous diagnostic copied this shape only into `config-undef6.h`, so later
`#include "config.h"` probes continued to measure a stale/generated header that
did not match the active derivation.

The diagnostic also adds smaller `config-undef6` `c-parse.c` prefix cuts at 40,
80, 120, 160, 200, 240, 280, 320, and 360 lines before the existing 390-line
header-only cut.  The next runtime run should distinguish whether the remaining
`config-undef6` crash is in the earliest Bison/front-matter block or a later
parser-table boundary.

## Verification

```sh
./target/debug/crunch eval bootstrap/diag-gcc40-c-parse-boundary.ncl
# exit 0

git diff --check
# exit 0
```

## Follow-up

Run the focused diagnostic through `crunch bootstrap validate` and record the new
`cparse_undef6_top_*` result matrix.  A fresh run was attempted with a new empty
state directory, but it was still rebuilding Mes prerequisites after 10 minutes;
that partial run produced only checkpoint files and was cleaned up without being
used as GCC boundary evidence.
