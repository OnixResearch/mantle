# GCC 4.0 rtl.c boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-gengtype-bridge-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-gengtype-bridge-evidence" \
  --resume
```

Result: build failed deterministically after the `gcc40-cc` generator bridge
advanced past the previous `gcc/gengtype.c` TinyCC segmentation boundary.
The bridge now stubs the `gengtype` objects and creates a bootstrap-only
`build/gengtype` script that emits empty generated `gt-*.h`, `gtype-desc.h`,
and `gtype-desc.c` artifacts.

Current boundary:

```text
gcc/rtl.c -> TinyCC Segmentation fault
```

Key log excerpt:

```text
gcc40-cc -c ... -o build/gengtype.o /tmp/gcc-build/gcc-4.0.4/gcc/gengtype.c
gcc40-cc -c ... -o build/gengtype-lex.o /tmp/gcc-build/gcc-4.0.4/gcc/gengtype-lex.c
gcc40-cc -c ... -o build/gengtype-yacc.o /tmp/gcc-build/gcc-4.0.4/gcc/gengtype-yacc.c
gcc40-cc ... -o build/gengtype ...
build/gengtype
gcc40-cc -c ... -o build/rtl.o /tmp/gcc-build/gcc-4.0.4/gcc/rtl.c
make: *** [build/rtl.o] Segmentation fault (core dumped)
```

Artifacts:

- `V2-gcc40-gcc-rtl-boundary-build.stdout.log`
- `V2-gcc40-gcc-rtl-boundary-build.stderr.log`
- `V2-gcc40-gcc-rtl-boundary-drv.log`

Validation summary:

- status: `build-failed`
- build_exit_code: `1`
- failure_class: `build`
- saved drv log: `/home/brittonr/.local/state/crunch/logs/lbn70ki0vrrqn8ir5ysd7jm7wfgb74d3-gcc-4.0.4.drv.log`
