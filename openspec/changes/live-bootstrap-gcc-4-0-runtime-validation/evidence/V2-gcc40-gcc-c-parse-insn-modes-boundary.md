# V2 gcc-4.0 `c-parse.c` generated `insn-modes.h` boundary

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/gcc40-cparse-insn-modes-boundary-r2-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-insn-modes-boundary-r2-state" \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Transcript: `evidence/V2-gcc40-gcc-c-parse-insn-modes-boundary-build.diag.log`.

The diagnostic intentionally exits nonzero because the full `c-parse.c` compile
still segfaults; the useful result is the ordered header probe boundary.

## Result

The previous machmode slice pointed at `machmode.h` line 26,
`#include "insn-modes.h"`. This run added focused probes for the generated
`insn-modes.h` artifact before continuing the existing machmode/tree probes.

Observed facts:

- `cparse_undef6_inc_tm` still passes: `rc=0`.
- The generated `insn-modes.h` found in the GCC build dir is a 22-line guarded
  enum header:
  - line 1: `#ifndef GCC_INSN_MODES_H`
  - line 5: `enum machine_mode {`
  - line 21: `};`
  - line 22: `#endif`
- A hand-written minimal enum equivalent passes:
  `cparse_undef6_insn_modes_minimal_enum rc=0`.
- Including the generated header directly now passes:
  `cparse_undef6_insn_modes_include_only rc=0`.
- The `machmode.h` include of `insn-modes.h` alone also passes:
  `cparse_undef6_machmode_insn_modes_only rc=0`.
- Complete machmode prefixes after the generated enum pass:
  `cparse_undef6_machmode_lines_{40,60,80,100,120} rc=0`.
- Raw incomplete prefix probes such as `insn_modes_lines_{1,5,10,20}` and the
  synthetic `balanced_{20,40}` probes can still crash, but those inputs are
  intentionally malformed partial preprocessor/enum fragments and are not the
  compiler-source boundary.
- Full `c-parse.c` remains at `rc=139`, as expected for this diagnostic.

## Interpretation

Generated `insn-modes.h` is not the next real parser blocker when compiled as a
complete header under the passing `config-undef6 + system.h + coretypes.h + tm.h`
context. The earlier machmode failure was too coarse: once the generated header
is captured and tested as a complete include, it passes in isolation and through
the minimal `machmode.h` include path.

The current narrow state is:

```c
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#include "tm.h"
#include "machmode.h"   /* with generated insn-modes.h included */
```

This path now passes through the `insn-modes.h` seam. The remaining `c-parse.c`
segfault is therefore beyond the complete generated machine-mode enum and should
be reduced at the next real frontend header construct rather than patching
`insn-modes.h`.

## Next probe

Continue after the cleared `insn-modes.h` seam: reduce the next `machmode.h` /
`tree.h` construct under the same sanitized context, starting with the next
included/generated mode metadata header or the first `tree.h` construct that
still reproduces `rc=139` with a complete, balanced probe.
