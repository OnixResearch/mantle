# V3 binutils subdir/libiberty boundary

Focused validation of `bootstrap/binutils-tcc.ncl` after switching from the
recursive top-level build to live-bootstrap-style per-subdirectory configure.

Command:

```sh
timeout 570 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/binutils-subdir9-store" \
  bootstrap validate bootstrap/binutils-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/binutils-subdir9-evidence" \
  --resume
```

Result:

```text
RC=1
build_exit_code=1
failure_class=build
```

Key observations:
- status: build-failed
- saved log: `/home/brittonr/.local/state/crunch/logs/ssk4pk997vv18gngws5v02kq35mkckbj-binutils-2.30-tcc.drv.log`
- observed: `configure: libiberty`
- observed: `configure: intl`
- observed: `configure: bfd`
- observed: `configure: opcodes`
- observed: `configure: binutils`
- observed: `configure: gas`
- observed: `configure: ld`
- observed: `build: libiberty`
- observed: `make: *** [fibheap.o] Segmentation fault`

The previous parent boundary was a recursive top-level `regex.o`/`bfd.lo`
segfault before a clean per-directory build sequence. This slice proves the
binutils release tree can now configure `libiberty`, `intl`, `bfd`, `opcodes`,
`binutils`, `gas`, and `ld` under the constrained TinyCC/musl toolchain and
advances the concrete compiler blocker to libiberty `fibheap.o` after local
source shims avoid earlier regex/md5/sha1/cplus-demangle/concat crashes.

The derivation is still incomplete: `gas/as` is not yet produced, and the next
fix should either repair or safely bypass libiberty `fibheap.c` compilation
without masking symbols needed by `gas`/`ld` runtime smoke tests.
