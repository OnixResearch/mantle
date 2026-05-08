# GCC 4.0 gengtype boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-gengenrtl-bridge-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-gengenrtl-bridge-evidence" \
  --resume
```

Result: build failed deterministically after the `gcc40-cc` generator bridge
advanced past the previous `gcc/gengenrtl.c` TinyCC segmentation boundary.
The bridge now stubs `gengenrtl.o` and creates a bootstrap-only
`build/gengenrtl` script that emits minimal `genrtl.h`/`genrtl.c` outputs.

Current boundary:

```text
gcc/gengtype.c -> TinyCC Segmentation fault
```

Key log excerpt:

```text
gcc40-cc -c ... -o build/gengenrtl.o /tmp/gcc-build/gcc-4.0.4/gcc/gengenrtl.c
gcc40-cc ... -o build/gengenrtl \
 build/gengenrtl.o build/errors.o ../build-x86_64-unknown-linux-gnu/libiberty/libiberty.a
build/gengenrtl -h > tmp-genrtl.h
build/gengenrtl > tmp-genrtl.c
gcc40-cc -c ... -o build/gengtype.o /tmp/gcc-build/gcc-4.0.4/gcc/gengtype.c
make: *** [build/gengtype.o] Segmentation fault (core dumped)
```

Artifacts:

- `V2-gcc40-gcc-gengtype-boundary-build.stdout.log`
- `V2-gcc40-gcc-gengtype-boundary-build.stderr.log`
- `V2-gcc40-gcc-gengtype-boundary-drv.log`

Validation summary:

- status: `build-failed`
- build_exit_code: `1`
- failure_class: `build`
- saved drv log: `/home/brittonr/.local/state/crunch/logs/195p8azgri69gvwzf1cqjg8kl5wa2nab-gcc-4.0.4.drv.log`
