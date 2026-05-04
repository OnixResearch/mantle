# tcc-musl link repair notes

Task-ID: V2, V3, V4
Covers: bootstrap.part.tcc.musl.runtime-validation

Focused validation of `bootstrap/tcc-musl.ncl` now passes. The accepted bridge:

- hosts TinyCC 0.9.27 object compilation with the known-good `tinycc-0.9.26` Mes compiler;
- ports the TinyCC source-normalization seams already proven in `tcc-musl-prep`;
- links the compiler executable with the Mes runtime objects/archive;
- installs `bin/tcc`, `bin/tcc-0.9.27-musl`, and `lib/tcc/libtcc1.a`;
- smoke-compiles a trivial C translation unit to an object with the installed compiler.

The earlier fully-linked smoke remains deferred to the downstream self-hosted musl-v2 stage because this first compiler is a bridge executable linked with Mes runtime but configured for musl headers/runtime paths.

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/tcc-musl-link7-store" \
  bootstrap validate bootstrap/tcc-musl.ncl \
  --evidence-dir "$PWD/.crunch-drain/tcc-musl-link7-evidence" \
  --resume
```

Result: pass (`build_exit_code: 0`, no leakage findings).
