# V2 gcc40 c-parse decl0 predecessor verbose/add-file diagnostic

Task-ID: V2 focused predecessor TinyCC verbose/add-file boundary follow-up for the GCC 4.0 c-parse `decl0` path.
Covers: bootstrap.gcc40.runtime-validation

## Commands

This follow-up ran a restored-output local replay from the same predecessor artifacts as the add-file-loop diagnostic:

```sh
openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-predecessor-verbose-20260510/run-local-probes.sh   > openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-predecessor-verbose-20260510/focused-local-probes.txt 2>&1
bash -n openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-predecessor-verbose-20260510/run-local-probes.sh
```

Replay inputs:

- TinyCC: `.crunch-drain/post621-restored-store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2/bin/tcc`
- musl headers/libs: `.crunch-drain/post621-restored-store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl`
- TinyCC source: `.crunch-drain/post621-restored-store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src`

## Focused results

| Probe | Result |
| --- | --- |
| `tiny base` (`int x;`, `-v`) | `rc=0`, object emitted, verbose prints corrupted `-> %s` / `<- %s` |
| `tiny boot` (diagnostic bootstrap flags) | `rc=0`, object emitted, same verbose corruption |
| `tiny full` (bootstrap + config path flags) | `rc=0`, object emitted, same verbose corruption |
| `h_only` (`#include "tcc.h"`) | `rc=0`, object emitted |
| `tcctools` / `h_tools` | `rc=0`, object emitted |
| `libtcc` | `rc=139`, no object emitted |
| `h_lib` / `h_lib_tools` | `rc=139`, no object emitted |

## Interpretation

The predecessor compiler's verbose filename display is already corrupted for a one-line successful compile, so `-> %s` is not by itself the fatal add-file boundary. The compile-only crash follows source content: normalized `tcc.h` and `tcctools.c` compile, while any one-source path that requires normalized `libtcc.c` still segfaults before object emission. Source-level add-file markers still cannot execute because the predecessor never emits the instrumented compiler object.

This narrows the next slice away from linker inputs and away from the public `tcc_add_file` call as the primary crash site. The next useful diagnostic is to reconcile the accumulated full-`libtcc.c` normalizations used by the earlier passing `libtcc.asm_simplified.c` replay with this one-source/predecessor compile script, then split the remaining missing normalization instead of treating verbose corruption as the crash cause.
