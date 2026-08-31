# ADR 0092: Frame Rust source identities with BLAKE3

## Status

Accepted (2026-08-31)

## Context

V89 passed checkpoint restoration, closure relocation, rustc compatibility, and
fixed-point preflight. Stage1 then entered native Rust planning.

Rust action planning rejected the first unit because its source algorithm was
`blake3-tree-v1`, not the literal string `blake3`.

Rust planning uses typed source identities:

- path and materialized vendor trees use `blake3-tree-v1`;
- Cargo registry sources retain Cargo checksum semantics;
- Git sources retain resolved revision semantics.

The action adapter copied the source value into a field named
`source_digest_blake3`, but first required one unsupported algorithm spelling.
This rejected real native plans and could not safely represent Cargo or Git
source identities.

## Decision Drivers

- Keep Mantle-owned action identities on BLAKE3.
- Preserve source-algorithm meaning instead of relabeling values.
- Retain Cargo SHA-256 only where Cargo interoperability requires it.
- Retain resolved Git revisions as source facts.
- Prevent equal values from different algorithms from aliasing.
- Keep the conversion pure, bounded, and deterministic.

## Decision

Validate that the source algorithm and value are nonempty bounded text without
NUL bytes.

Construct one framed value:

```text
algorithm NUL value
```

Hash that frame with BLAKE3 under the domain
`mantle-source-built-rust-source-identity-v1`.

Store the result as the unit action's `source_digest_blake3`. The action plan,
input authority, and output identity continue to consume a 64-character BLAKE3
value.

Do not rewrite Cargo checksums, Git revisions, or tree-digest algorithms to the
string `blake3`. Their typed values remain visible in Rust planning receipts.
Only the action adapter derives its own canonical BLAKE3 identity.

## Alternatives Considered

### Accept `blake3-tree-v1` as an alias for `blake3`

Rejected. This would still reject Cargo and Git sources and would erase domain
meaning.

### Copy every source value without validation

Rejected. SHA-256 checksums and Git revisions are not BLAKE3 digests.

### Rehash only non-BLAKE3 algorithms

Rejected. Conditional behavior can alias a raw digest with a framed identity.
Every source identity uses the same framing rule.

### Replace Cargo SHA-256 with BLAKE3

Rejected. Cargo checksum metadata is an interoperability surface that requires
SHA-256.

## Consequences

- Path, registry, and Git units receive BLAKE3 action identities.
- Equal source values with different algorithms produce different identities.
- Empty or malformed source identities fail before action planning.
- Rust planning receipts preserve their original source semantics.
- V89 remains failed evidence. A fresh promoted proof must verify this adapter.
