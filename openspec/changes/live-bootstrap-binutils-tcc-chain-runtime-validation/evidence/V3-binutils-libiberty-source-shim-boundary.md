# V3 binutils libiberty source-shim boundary

Focused validation of `bootstrap/binutils-tcc.ncl` after extending the
libiberty source shims from object placeholders to overwriting problematic
source files before `make`.

Command:

```sh
timeout 570 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/binutils-srcshim-store" \
  bootstrap validate bootstrap/binutils-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/binutils-srcshim-evidence" \
  --resume
```

Result:

```text
RC=1
build_exit_code=1
failure_class=build
saved_log_path=/home/brittonr/.local/state/crunch/logs/qnzxyxjxkiz99aj5k767mfkp780916zc-binutils-2.30-tcc.drv.log
```

Key observations:

- `build: libiberty`
- `make: *** [xasprintf.o] Segmentation fault`
- `configure: gas`
- `configure: ld`

The previous committed boundary stopped at libiberty `fibheap.o`. This follow-up
shows source-file shims move the TinyCC/musl libiberty compiler crash past
`fibheap`, `getopt*`, `pex*`, and related auxiliary files. The current concrete
blocker is now `xasprintf.o`; `gas/as` remains unbuilt.
