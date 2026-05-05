# GCC 4.0 c-parse boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-gencheck-bridge-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-gencheck-bridge-evidence" \
  --resume
```

Result: build failed deterministically after the GCC 4.0 generator bridge advanced
past the previous `genconstants` boundary and the subsequent generator/support
compile boundaries (`genflags`, `genconditions`, `genpreds`, `print-rtl`, and
`gencheck`). The bootstrap wrapper now creates build-only generator stubs for
those TinyCC-hostile generator paths and emits minimal generated headers/sources
needed to reach the first non-generator compiler source boundary.

Current boundary:

```text
gcc/c-parse.c -> TinyCC Segmentation fault
```

Key log excerpt:

```text
build/gencheck > tmp-check.h
/bin/sh /tmp/gcc-build/gcc-4.0.4/gcc/../move-if-change tmp-check.h tree-check.h
echo timestamp > s-check
gcc40-cc -c ... /tmp/gcc-build/gcc-4.0.4/gcc/c-parse.c -o c-parse.o
make: *** [c-parse.o] Segmentation fault (core dumped)
```

Artifacts:

- `V2-gcc40-gcc-c-parse-boundary-build.stdout.log`
- `V2-gcc40-gcc-c-parse-boundary-build.stderr.log`
- `V2-gcc40-gcc-c-parse-boundary-wrapper.stdout.log`
- `V2-gcc40-gcc-c-parse-boundary-wrapper.stderr.log`
- `V2-gcc40-gcc-c-parse-boundary-drv.log`

Validation summary:

- status: `build-failed`
- build_exit_code: `1`
- failure_class: `build`
- saved drv log: `/home/brittonr/.local/state/crunch/logs/b54wjlwksv1jwc5rkjjzqnyi7ag734bg-gcc-4.0.4.drv.log`
