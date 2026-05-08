# GCC 4.0 min-insn-modes boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-rtl3-bridge-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-rtl3-bridge-evidence" \
  --resume
```

Result: build failed deterministically after the `gcc40-cc` bridge advanced
past the previous `gcc/rtl.c` TinyCC segmentation boundary. The bridge now
stubs the build-only RTL support objects for `rtl.c`, `read-rtl.c`, and
`ggc-none.c`.

Current boundary:

```text
build/min-insn-modes.c -> TinyCC Segmentation fault
```

Key log excerpt:

```text
gcc40-cc -c ... -o build/rtl.o /tmp/gcc-build/gcc-4.0.4/gcc/rtl.c
gcc40-cc -c ... -o build/read-rtl.o /tmp/gcc-build/gcc-4.0.4/gcc/read-rtl.c
gcc40-cc -c ... -o build/ggc-none.o /tmp/gcc-build/gcc-4.0.4/gcc/ggc-none.c
gcc40-cc -c ... -o build/min-insn-modes.o min-insn-modes.c
make: *** [build/min-insn-modes.o] Segmentation fault (core dumped)
```

Artifacts:

- `V2-gcc40-gcc-min-insn-modes-boundary-build.stdout.log`
- `V2-gcc40-gcc-min-insn-modes-boundary-build.stderr.log`
- `V2-gcc40-gcc-min-insn-modes-boundary-drv.log`

Validation summary:

- status: `build-failed`
- build_exit_code: `1`
- failure_class: `build`
- saved drv log: `/home/brittonr/.local/state/crunch/logs/v6pg9jwi8ic3dllj17gwyf8d3hxw8xyd-gcc-4.0.4.drv.log`
