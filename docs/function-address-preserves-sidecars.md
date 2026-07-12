# Function-address Preserves sidecars

Mantle can derive its Cairn-ready function-address receipt from a typed opaque-evidence binding whose canonical artifact is a Kamacite Preserves receipt.

```sh
mantle release function-address-bind ./release-bundle \
  --mode required \
  --from-preserves-binding \
  --receipt-out ./function-address-binding.json
```

`--from-preserves-binding` is manifest-driven. It conflicts with explicit `--sidecar`, `--valence-receipt`, `--kamacite-receipt`, and `--release-binary` selectors. Mantle requires exactly one valid `function-address-preserves-v1` binding in the verified release manifest.

## Profile

The profile binds:

- canonical role `kamacite-function-address-preserves-receipt`;
- canonical schema `kamacite.function-address-preserves-receipt.v1`;
- canonical Preserves BLAKE3 and bounded byte size;
- Valence role/schema, JSON artifact digest, and logical `receipt_hash`;
- release source archive and binary digests;
- upstream and Mantle policy hashes;
- identity/linkage-only claim scope and explicit non-claims;
- optionally, one JSON compatibility projection.

The optional projection has its own artifact digest. Its public logical `receipt_hash` must equal the canonical Preserves BLAKE3 identity. It is never authoritative.

## Verification boundary

Mantle verifies the release bundle first, then reopens every selected artifact without following symlinks. Each reopened file is read through a fixed byte limit and rehashed against the verified manifest before identity extraction. Canonical Preserves bytes are never parsed. Only the bounded public fields of the Valence receipt and optional JSON projection are decoded.

The generated `mantle.function-address-binding.v1` receipt preserves two identity domains:

- `*_receipt_digest_blake3` fields identify bundle-local file bytes;
- `*_receipt_hash_blake3` fields identify public logical receipts.

For a Preserves-backed binding, the sidecar identity is the canonical Preserves BLAKE3. In required mode the optional Kamacite fields are populated only when the JSON projection exists; optional mode remains valid without that projection.

## Failure behavior

Missing or duplicate typed bindings, stale bytes, digest or size drift, wrong roles or schemas, stale Valence links, projection drift, source/binary mismatch, malformed public envelopes, overclaims, and oversized reads fail closed before output. Existing output paths are never overwritten. A structurally complete policy `FAIL` receipt may be retained only when the evidence bytes and identities are otherwise valid.

This boundary proves bounded artifact identity and release linkage only. It does not prove function semantics, Rust behavior, Kamacite or Valence soundness, whole-program safety, build correctness, or release eligibility.
