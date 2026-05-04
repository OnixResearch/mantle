# V2 build

Task-ID: V2
Covers: bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation

Status: captured.

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/musl-tcc-musl3-store" \
  bootstrap validate bootstrap/musl-1.1.24-tcc-musl.ncl \
  --evidence-dir "$PWD/.crunch-drain/musl-tcc-musl3-evidence" \
  --resume
```

Result: pass (`build_exit_code: 0`). Evidence prefix: `V2-musl-1.1.24-tcc-musl-pass-*`.

Implementation note: this second musl pass reuses the first-musl source compatibility bridge with `tcc-0.9.27-musl` as compiler, adds the previous musl include path, and adds a Makefile `CC_CMD` chmod hook so TinyCC-created objects are readable for follow-on copy/archive rules.
