# V2 build

After the `sed-tcc`/`tcc-musl-v2` prerequisite chain was archived, the bzip2 musl part was rerun with focused validation. The earlier blocker was replaced by a successful direct build.

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/bzip2-musl15-store" \
  bootstrap validate bootstrap/bzip2-1.0.8-musl.ncl \
  --evidence-dir "$PWD/.crunch-drain/bzip2-musl15-evidence" \
  --resume
```

Result:

- status: `passed`
- build exit code: `0`
- output: `/home/brittonr/git/crunch/crunch/.crunch-drain/bzip2-musl15-store/xwwwxypc42pncdzl4bj46cwjw4lzdhfz-bzip2-1.0.8-musl`
- logical output: `/crunch/store/xwwwxypc42pncdzl4bj46cwjw4lzdhfz-bzip2-1.0.8-musl`
- copied validation artifacts: `evidence/V2-bzip2-musl-pass-*`

Implementation notes for this pass:

- `tcc-musl` and `tcc-musl-v2` now avoid Mes/TinyCC `sprintf` for `tcctools.c` archive size fields, so downstream static archives are readable.
- `musl-1.1.24-tcc-musl` now ships the minimal runtime stubs needed by bzip2/bzip2recover static links.
- `bzip2-1.0.8-musl` compiles objects with the stable TinyCC 0.9.26 host path and links explicitly against declared musl/libtcc1 artifacts instead of relying on fragile `-L/-l` lookup.
