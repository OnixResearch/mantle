# Signed release witness handoff

This Rust example constructs canonical release and witness attestations, signs each with a distinct Ed25519 key, copies only the witness sidecars into the publisher verification directory, discovers the resulting directory, verifies both signatures, and evaluates a one-witness policy.

Run from the repository root:

```sh
cargo run -p mantle --example release_witness_handoff
```

The negative paths reject an unknown signature key, a witness bound to the wrong release, an insufficient witness set, and a revoked witness. The `mantle-project.ncl` package admits the exact BLAKE3-fixed source for inspection.

The release key is a public test fixture and the witness key is ephemeral. This example proves bounded signature and policy behavior over synthetic attestations; it is not a release-evidence bundle, independent rebuild, or production trust ceremony.
