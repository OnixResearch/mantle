# Design: Native vendor-deps source layout planning

## Source root discovery

The native registry source planner already reads `.cargo/config{,.toml}` source replacement directories. This change also treats a checkout-local `vendor-deps/` directory as explicit local source material when it exists.

## Package layout binding

For each Cargo.lock registry package, planning checks both supported local layouts under each vendor root:

1. `<vendor-root>/<name>-<version>/Cargo.toml`
2. `<vendor-root>/<name>/Cargo.toml`

The selected manifest remains checksum-bound through `.cargo-checksum.json` and source-tree digest evidence.

## Oracle comparison

Cargo metadata reports registry package manifests under Cargo's cache even when native planning binds a local vendor root. Native package comparison therefore accepts exact package ID matches before falling back to manifest-path equality. This keeps Cargo as oracle evidence without depending on Cargo cache paths.

## Failure behavior

Missing vendor roots, missing manifests, bad checksums, and unreadable source trees remain deterministic blockers. The planner must not use network/index access or `$CARGO_HOME` as a source fallback.
