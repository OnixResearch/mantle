# Design: Valence-validated release provenance

## Approach

Mantle remains an opaque-bundle verifier for stack provenance sidecars. The core
release verifier checks declared release metadata, sidecar bytes/digests,
Valence verification receipt presence, receipt hash binding, and binary identity
linkage. It does not parse or reinterpret Valence stack semantics.

## Policy shape

A release profile can declare stack provenance as optional or required. When
required, release verification must find:

- external evidence role `stack-provenance-trace`;
- expected Valence stack-provenance schema and claim scope;
- sidecar BLAKE3 digest matching bundled bytes;
- Valence verification receipt reference and BLAKE3 digest;
- release binary identity matching the sidecar/receipt metadata;
- explicit non-claims that Mantle validates bundle-local evidence only.

Human-authored policy should be Nickel-authored and exported to deterministic
runtime input where needed.

## Bundle binding

The release bundle records the sidecar artifact and the Valence verification
receipt as separate artifacts. The release verification report links them by safe
relative path, role, schema, claim scope, BLAKE3 digest, and binary identity. The
report can say the bundle carries Valence-validated stack provenance; it must not
claim Mantle performed semantic stack verification.

## Failure model

Required mode fails closed for missing sidecar, wrong role, wrong schema, wrong
claim scope, stale sidecar digest, missing Valence receipt, stale receipt digest,
missing binary identity, or weakened non-claim boundary. Optional mode records a
skipped or absent disposition without treating the release bundle as carrying
stack provenance.

## Fixture strategy

Positive fixtures cover optional absent sidecar, optional present sidecar, and
required present sidecar with matching Valence receipt. Negative fixtures cover
each required-mode failure plus an overclaiming non-claim boundary.

## What is explicitly not provided

- No Valence semantics implementation in Mantle.
- No source-code correctness proof.
- No proof that the Valence verifier itself is sound.
- No remote publication, signature, credential, or mirror-trust claim.
