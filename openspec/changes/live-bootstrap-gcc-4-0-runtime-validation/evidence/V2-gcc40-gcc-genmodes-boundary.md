# GCC 4.0 manual libcpp / genmodes boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-manual-libcpp-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-manual-libcpp-evidence" \
  --resume
```

Result: build failed deterministically after advancing past the previous
`libcpp/charset.c` TinyCC segmentation boundary.

Observed progress:

- configured `libiberty`, `libcpp`, and `gcc`;
- built reduced `libiberty.a`;
- built reduced `libcpp.a` from bootstrap shims;
- entered `make -C build/gcc`;
- generated `config.h`, `tm.h`, and `bconfig.h`;
- stopped at the first GCC generator compile boundary:

```text
gcc40-cc ... -o build/genmodes.o /tmp/gcc-build/gcc-4.0.4/gcc/genmodes.c
make: *** [build/genmodes.o] Segmentation fault (core dumped)
```

Validation summary: `V2-gcc40-gcc-genmodes-boundary-validation-summary.json`.
Build transcript: `V2-gcc40-gcc-genmodes-boundary-build.stdout.log`.
Driver log: `V2-gcc40-gcc-genmodes-boundary-drv.log`.

This is still not full GCC runtime validation. The `libcpp` archive is a narrowed
bootstrap bridge used to expose the next deterministic boundary.
