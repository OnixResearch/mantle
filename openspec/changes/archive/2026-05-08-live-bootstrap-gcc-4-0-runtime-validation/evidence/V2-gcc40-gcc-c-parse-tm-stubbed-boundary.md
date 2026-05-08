# GCC 4.0 c-parse tm.h stubbed boundary

## Command

Diagnostic reuse run:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store $PWD/.crunch-drain/gcc40-tm-head/store \
  --state-dir $PWD/.crunch-drain/gcc40-tm-head/state \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Main target reuse run:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store $PWD/.crunch-drain/gcc40-tm-head/store \
  --state-dir $PWD/.crunch-drain/gcc40-tm-head/state \
  bootstrap/gcc-4.0.ncl
```

## Result

- Diagnostic command completed with exit code `1` because the diagnostic target intentionally returns failure after probing remaining crashes.
- Main `bootstrap/gcc-4.0.ncl` completed with exit code `1`, but advanced past the prior `tm.h`/missing generator-header crash.
- `bootstrap/gcc-4.0.ncl` now seeds `gcc/insn-constants.h` and `gcc/insn-flags.h` with the same safe stubs as the generator bridges before `tm.h` can include them.

## Boundary evidence

After config undef6 plus seeded generator headers:

```text
diag-cparse: cparse_undef6_inc_tm rc=0
diag-cparse: cparse_undef6_insn_constants_only rc=0
diag-cparse: cparse_undef6_insn_flags_only rc=0
diag-cparse: cparse_undef6_tm_generator_file rc=0
diag-cparse: cparse_undef6_tm_used_for_target rc=0
diag-cparse: cparse_undef6_tm_undef_ingcc rc=0
diag-cparse: cparse_undef6_tm_first_block_insn_both rc=0
```

The generated `tm.h` head is now the expected complete include wrapper:

```text
#ifndef GCC_TM_H
#define GCC_TM_H
#ifdef IN_GCC
# include "config/i386/biarch64.h"
# include "config/i386/i386.h"
# include "config/i386/unix.h"
# include "config/i386/att.h"
# include "config/dbxelf.h"
# include "config/elfos.h"
# include "config/svr4.h"
# include "config/linux.h"
# include "config/i386/x86-64.h"
# include "config/i386/linux64.h"
# include "defaults.h"
#endif
#if defined IN_GCC && !defined GENERATOR_FILE && !defined USED_FOR_TARGET
# include "insn-constants.h"
# include "insn-flags.h"
#endif
#endif /* GCC_TM_H */
```

The main target reaches the real `c-parse.c` compile boundary:

```text
gcc40-cc -c ... /tmp/gcc-build/gcc-4.0.4/gcc/c-parse.c -o c-parse.o
make: *** [c-parse.o] Segmentation fault (core dumped)
```

## Artifacts

- Diagnostic extract: `V2-gcc40-gcc-c-parse-tm-stubbed-boundary-diag.log`
- Main build tail: `V2-gcc40-gcc-c-parse-tm-stubbed-boundary-build.log`

## Interpretation

This slice removes the `tm.h`/missing `insn-constants.h` and `insn-flags.h` crash as the active blocker. The remaining blocker is no longer target-header setup; it is TinyCC segfaulting while compiling the full generated GCC 4.0 `c-parse.c` translation unit after all generator/header setup succeeds.
