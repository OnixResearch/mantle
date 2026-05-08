# GCC 4.0 genmddeps boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-genmddeps-boundary-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-genmddeps-boundary-evidence" \
  --resume
```

Result: build failed deterministically after the `gcc40-cc` generator bridge
advanced past the previous `gcc/genmodes.c` TinyCC segmentation boundary.
The bridge creates bootstrap-only `genmodes`/`errors` artifacts sufficient for
`make -C build/gcc` to generate `insn-modes.h`, `min-insn-modes.c`, and
`insn-modes.c`.

Current boundary:

```text
gcc/genmddeps.c -> TinyCC Segmentation fault
```

Key log excerpt:

```text
gcc40-cc -c ... -o build/genmodes.o /tmp/gcc-build/gcc-4.0.4/gcc/genmodes.c
gcc40-cc -c ... -o build/errors.o /tmp/gcc-build/gcc-4.0.4/gcc/errors.c
gcc40-cc ... -o build/genmodes \
 build/genmodes.o build/errors.o ../build-x86_64-unknown-linux-gnu/libiberty/libiberty.a
build/genmodes -h > tmp-modes.h
/bin/sh /tmp/gcc-build/gcc-4.0.4/gcc/../move-if-change tmp-modes.h insn-modes.h
build/genmodes -m > tmp-min-modes.c
/bin/sh /tmp/gcc-build/gcc-4.0.4/gcc/../move-if-change tmp-min-modes.c min-insn-modes.c
build/genmodes > tmp-modes.c
/bin/sh /tmp/gcc-build/gcc-4.0.4/gcc/../move-if-change tmp-modes.c insn-modes.c
echo timestamp > s-modes
gcc40-cc -c ... -o build/genmddeps.o /tmp/gcc-build/gcc-4.0.4/gcc/genmddeps.c
make: *** [build/genmddeps.o] Segmentation fault (core dumped)
```

Artifacts:

- `V2-gcc40-gcc-genmddeps-boundary-build.stdout.log`
- `V2-gcc40-gcc-genmddeps-boundary-build.stderr.log`
- `V2-gcc40-gcc-genmddeps-boundary-drv.log`

Validation summary:

- status: `build-failed`
- build_exit_code: `1`
- failure_class: `build`
- saved drv log: `/home/brittonr/.local/state/crunch/logs/5i6db39wdjfplr5jj4lnvfvnpjgqmfxh-gcc-4.0.4.drv.log`
