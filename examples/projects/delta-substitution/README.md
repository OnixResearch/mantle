# Delta substitution adaptor

The Rust example exercises Mantle's shipped delta substitution adaptor with three outcomes:

- a receiver reuses the first v1 chunk and transfers only the changed v2 chunk;
- an authority without delta capability falls back to the full artifact;
- missing sender chunk data fails closed.

Run from the repository root:

```sh
cargo run -p mantle --example delta_substitution
```

The `mantle-project.ncl` package independently admits the exact BLAKE3-fixed example source:

```sh
cd examples/projects/delta-substitution
mantle build .#source
```

This uses the in-memory authority adaptor and real signature, manifest, transfer, attestation, and policy paths. It does **not** claim that an HTTP cache server was contacted; production HTTP endpoint behavior remains covered by the store integration rails.
