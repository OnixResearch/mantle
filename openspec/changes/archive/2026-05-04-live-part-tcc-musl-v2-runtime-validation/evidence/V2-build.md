# V2 build

Task-ID: V2
Covers: bootstrap.part.tcc.musl.v2.runtime-validation

Status: captured.

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/tcc-musl-v2c-store" \
  bootstrap validate bootstrap/tcc-musl-v2.ncl \
  --evidence-dir "$PWD/.crunch-drain/tcc-musl-v2c-evidence" \
  --resume
```

Result: pass (`build_exit_code: 0`). Evidence prefix: `V2-tcc-musl-v2-pass-*`.

Implementation note: the v2 compiler now reuses the validated `tcc-musl` bridge shape while targeting the second-pass musl: TinyCC 0.9.26 hosts the TinyCC 0.9.27 source build, the prep source normalizations are applied, the compiler is linked explicitly with Mes runtime objects/archive, `libtcc1.a` is installed, and the installed compiler smoke-compiles a trivial C translation unit to an object.
