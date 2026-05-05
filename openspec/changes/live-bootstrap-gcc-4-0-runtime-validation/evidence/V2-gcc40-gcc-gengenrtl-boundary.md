# GCC 4.0 gengenrtl boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-genmddeps-bridge-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-genmddeps-bridge-evidence" \
  --resume
```

Result: build failed deterministically after the `gcc40-cc` generator bridge
advanced past the previous `gcc/genmddeps.c` TinyCC segmentation boundary.
The bridge now stubs `genmddeps.o` and creates a bootstrap-only
`build/genmddeps` script that emits an empty `MD_INCLUDES` fragment.

Current boundary:

```text
gcc/gengenrtl.c -> TinyCC Segmentation fault
```

Key log excerpt:

```text
gcc40-cc -c ... -o build/genmddeps.o /tmp/gcc-build/gcc-4.0.4/gcc/genmddeps.c
gcc40-cc -c ... -o build/gengenrtl.o /tmp/gcc-build/gcc-4.0.4/gcc/gengenrtl.c
make: *** [build/gengenrtl.o] Segmentation fault (core dumped)
```

Artifacts:

- `V2-gcc40-gcc-gengenrtl-boundary-build.stdout.log`
- `V2-gcc40-gcc-gengenrtl-boundary-build.stderr.log`
- `V2-gcc40-gcc-gengenrtl-boundary-drv.log`

Validation summary:

- status: `build-failed`
- build_exit_code: `1`
- failure_class: `build`
- saved drv log: `/home/brittonr/.local/state/crunch/logs/p5s4xfg1wn2fajqyw2d8bcmylqmz8rk2-gcc-4.0.4.drv.log`
