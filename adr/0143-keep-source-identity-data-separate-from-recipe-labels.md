# ADR 0143: Keep source identity data separate from recipe labels

## Status

Accepted 2026-11-06 for successor source-program authoring. This does not accept
provider execution, a fixed point, or a release.

## Context

Real preparation passed successor dependency resolution and reached recipe-label
admission. The generator interpolated package versions and crate-type lists
into step IDs. Cargo build metadata contains `+`; a combined Rust crate type is
`rlib,dylib`. Neither character belongs to the recipe label grammar. Rewriting
these data values themselves would change Cargo metadata or compiler behavior.

Package-name punctuation also cannot be normalized lossily when it contributes
to producer identity. Different input identities must not accidentally share an
output authority or compiler metadata label.

## Decision Drivers

- Preserve original Cargo names/versions and compiler crate-type arguments.
- Keep the existing recipe grammar, 256-byte label ceiling and producer checks.
- Preserve distinct producer identities, including literal escape-like text.
- Reuse one small pure projection for generated label components, not a global
  post-hoc rewrite of IDs and already-created producer references.
- Add no replacement digests, compatibility aliases, truncation or label bypass.

## Decision

Encode data components before constructing successor unit and library step IDs.
Keep ASCII alphanumeric bytes, hyphens and dots. Encode every other UTF-8 byte
as an underscore followed by two lowercase hexadecimal digits. An underscore
is itself encoded, so literal escape-looking input cannot collide with an
encoded character. No decoding is needed by the executor.

Apply this projection to package names, package versions and crate-type label
components. The original values remain in source metadata and compiler argument
construction. The full generated ID still goes through ordinary recipe
validation; encoding does not truncate an oversized ID or relax any grammar.

Already-safe components remain borrowed. Otherwise, the implementation measures
exact encoded capacity before its one allocation.
It is a label projection, not a new content-identity algorithm: existing
BLAKE3-bound recipes, producer links and observed artifacts retain authority.

## Consequences

Mantle's Rust source-authoring owners maintain this boundary. Producer labels
and derived compiler metadata names change where unsafe bytes or underscores
were previously interpolated/normalized, so regenerated programs require their
new source/profile identities. Old prepared programs are not silently relabeled.

Regression fixtures exercise real recipe admission: raw build-metadata and
combined-kind labels are rejected; encoded labels are admitted; metadata versus
prerelease versions and comma versus literal escape text keep distinct produced
references. Runtime preparation remains the integration check. It is not proof
of original compiler execution or fixed-point equality.
