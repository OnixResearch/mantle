# V2 gcc-4.0 `c-parse.c` machmode boundary

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/gcc40-cparse-machmode-boundary-r3-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-machmode-boundary-r3-state" \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Transcript: `evidence/V2-gcc40-gcc-c-parse-machmode-boundary-build.diag.log`.

The diagnostic intentionally exits nonzero because the full `c-parse.c` compile still
segfaults; the useful result is the ordered probe boundary in the log.

## Result

The previous tree boundary showed that `tree.h` line 25, `#include "machmode.h"`,
was the first crashing line under the sanitized `config-undef6 + system.h +
coretypes.h + tm.h` frontend context. This run moved the probe into generated
`machmode.h`.

Observed narrow boundary:

- `cparse_undef6_inc_tm` passes: `rc=0`.
- The generated `machmode.h` head starts with the include guard at lines 22-23.
- `machmode.h` guard-only passes: `cparse_undef6_machmode_guard_only rc=0`.
- The first semantic include in `machmode.h` fails:
  `cparse_undef6_machmode_insn_modes_only rc=139`.
- Any balanced `machmode.h` prefix that reaches the generated header body also
  fails: `cparse_undef6_machmode_lines_{20,40,60,80,100,120} rc=139`.
- The prior tree result is reproduced in the same run:
  `cparse_undef6_tree_guard_only rc=0`,
  `cparse_undef6_tree_machmode_only rc=139`, and
  `cparse_undef6_tree_lines_25 rc=139`.
- Full `c-parse.c` remains at `rc=139`, as expected for this diagnostic.

## Interpretation

The frontend parser crash is now below `tree.h` and at the generated machine-mode
header seam. Under the passing `config-undef6 + system/coretypes/tm` context, the
minimal machmode reproducer is effectively:

```c
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#include "tm.h"
#include "insn-modes.h"
int machmode_manual_undef6_probe;
```

`machmode.h` line 26 (`#include "insn-modes.h"`) is the first failing construct;
the plain include guard alone does not fail.

## Next probe

Reduce generated `insn-modes.h` under the same `config-undef6 + system.h +
coretypes.h + tm.h` context. The next useful slice is to print the generated
`insn-modes.h` head and test guard-only / enum-prefix / first `enum machine_mode`
entries before attempting a compiler source patch.
