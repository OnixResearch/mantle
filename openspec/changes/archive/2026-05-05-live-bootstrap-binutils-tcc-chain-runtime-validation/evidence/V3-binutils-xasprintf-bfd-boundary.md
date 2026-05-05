# V3 binutils xasprintf to BFD boundary evidence

Task-ID: V3
Covers: live-bootstrap-binutils-tcc-chain-runtime-validation parent binutils build
Status: captured

## Command

```sh
timeout 570 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/binutils-xasprintf-store" \
  bootstrap validate bootstrap/binutils-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/binutils-xasprintf-evidence" \
  --resume
```

## Result

- runner exit: `1`
- build status: `build-failed`
- saved derivation log: `/home/brittonr/.local/state/crunch/logs/f695xs79gnkcic9qns3qcgib8nlwd1py-binutils-2.30-tcc.drv.log`
- copied evidence prefix: `V3-binutils-xasprintf-bfd-boundary-*`

## Observed progress

The source-level `libiberty/xasprintf.c` shim moved the parent binutils build past the previous `make: *** [xasprintf.o] Segmentation fault` blocker. The validation built `libiberty` and entered the `bfd` subdirectory build.

## Current blocker

The next concrete blocker is the BFD compile boundary:

```text
build: bfd
make: the `-l' option requires a positive integral argument
make: the `-l' option requires a positive integral argument
make: the `-l' option requires a positive integral argument
Segmentation fault (core dumped)
make[2]: *** [bfd.lo] Error 1
make[1]: *** [all-recursive] Error 1
make: *** [all] Error 2
ERROR: build failed in bfd
```

`gas/as` is still not produced; V4/V5 remain open until a successful parent build transcript exists.
