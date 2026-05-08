# GCC 4.0 genconstants boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-gensupport-bridge-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-gensupport-bridge-evidence" \
  --resume
```

Result: build failed deterministically after the `gcc40-cc` bridge advanced
past the previous `build/min-insn-modes.c` TinyCC segmentation boundary. The
bridge now stubs `min-insn-modes.c` and `gensupport.c` for the build-only
generator support path; the existing `genmddeps` bridge still emits `mddeps.mk`.

Current boundary:

```text
gcc/genconstants.c -> TinyCC Segmentation fault
```

Key log excerpt:

```text
gcc40-cc -c ... -o build/min-insn-modes.o min-insn-modes.c
gcc40-cc -c ... -o build/gensupport.o /tmp/gcc-build/gcc-4.0.4/gcc/gensupport.c
gcc40-cc -c ... -o build/dummy-conditions.o /tmp/gcc-build/gcc-4.0.4/gcc/dummy-conditions.c
gcc40-cc ... -o build/genmddeps ...
build/genmddeps /tmp/gcc-build/gcc-4.0.4/gcc/config/i386/i386.md > tmp-mddeps
gcc40-cc -c ... -o build/genconstants.o /tmp/gcc-build/gcc-4.0.4/gcc/genconstants.c
make: *** [build/genconstants.o] Segmentation fault (core dumped)
```

Artifacts:

- `V2-gcc40-gcc-genconstants-boundary-build.stdout.log`
- `V2-gcc40-gcc-genconstants-boundary-build.stderr.log`
- `V2-gcc40-gcc-genconstants-boundary-drv.log`

Validation summary:

- status: `build-failed`
- build_exit_code: `1`
- failure_class: `build`
- saved drv log: `/home/brittonr/.local/state/crunch/logs/jjca745x4bj4hz29pdnm3sbl22arfi1d-gcc-4.0.4.drv.log`
