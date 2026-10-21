# ADR 0122: Use native BLAKE3 streaming at the Snix boundary

- **Status:** Proposed
- **Date:** 2026-10-20

## Context

Mantle's declared Basalt authority SDK at immutable revision
`4c5c9e919b4573a7865559db2dc98a99c0c58144` includes UCAN packages requiring
BLAKE3 `=1.8.5` and zeroize `=1.9.0`. Actual dependency-resolution attempts
1393 and 1396 on Leviathan failed against Mantle's older lock and BLAKE3
`=1.8.2` constraints. This failure precedes compilation; a source-only API
review of the SDK does not establish consumer build compatibility.

The older BLAKE3 pin protected an unstable trait boundary: Snix castore's
`B3HashingReader` was an alias of a generic `digest::Digest` reader, and its
output conversion named `digest::Output<blake3::Hasher>`. BLAKE3's newer
`traits-preview` implementation uses a different `digest` generation from
Mantle's SHA/Nix interoperability dependencies.

## Decision drivers

- Use the real published authority SDK, not a local shadow implementation.
- Resolve one reviewed BLAKE3 version without changing content identities.
- Preserve the SHA/Nix `digest` boundary and avoid global cryptographic API churn.
- Preserve streaming, partial reads and pinning without extra allocation or copying.
- Keep the adapted Snix fork's provenance and verification scope explicit.

## Decision

Pin first-party BLAKE3 consumers and Snix castore to `=1.8.5`. Regenerate the
Cargo lock and source maps through Cargo and the existing repository/Nix
mechanisms. Retain `digest =0.10.7` for its existing SHA/Nix consumers; satisfy
the SDK's zeroize requirement through the generated lock.

Use `blake3::Hasher` directly in `B3HashingReader`, returning `blake3::Hash`.
Keep the generic `HashingReader<R, H>` for its existing `digest` consumers.
Remove BLAKE3's `traits-preview` feature and the obsolete digest-output
conversion. Existing consumers convert the native hash through
`B3Digest::from(blake3::Hash)`; no compatibility alias, replacement algorithm,
second BLAKE3 source or fallback hash is admitted.

A read hashes only newly filled bytes, never an existing `ReadBuf` prefix.
Pending/error behavior and the wrapped reader's result remain unchanged.
The stream reader owns the existing hasher and pinned reader; the cutover
must not buffer the input or add a second content pass.

This is a Mantle-local adaptation under ADR 0054, not an upstream Snix
backport or a bulk import. The source tree does not have one known upstream
Snix revision. The pre-change integrated source snapshot is
`325b35772740433c54623bca33b474aba54d924b393e170a00b54e7e2792bfe2`,
with base Mantle revision `43ccd68840b748e03773fc1bff5eb1cfca99dffa`.
Its measured BLAKE3 preimages are:

| Local file | BLAKE3 |
|---|---|
| `vendor/snix-castore/Cargo.toml` | `f6b19e2d71187e1937de9fdde82ec976ae1521bc8d0d56222908ac73de023e6b` |
| `vendor/snix-castore/src/hashing_reader.rs` | `fc52aef3a0db00fe282444dd0240187b4114651961820599aea33be53651cae2` |
| `vendor/snix-castore/src/digests.rs` | `dc2855866e764ce1d7959105f316969231a62491bb67602582ff8e7aa9ff5a4b` |

The final source manifest and acceptance transcripts must bind the postimage.
Historical fixtures, source receipts and immutable bootstrap inputs are not
rewritten to suggest they used the new version.

## Consequences

Mantle's store maintainers own the native streaming adaptation; dependency
maintainers own the common pin, generated lock and source-map regeneration.
The immediate consumer is the Basalt-backed Mantle gateway authority path.
The durable contribution is removal of BLAKE3's unstable trait coupling, not
a new cryptographic abstraction or an authority-policy change.

Acceptance requires the existing empty/small/large streaming cases, a
prefilled/partial-read boundary regression, the ordinary first-party gates,
and an actual store/build round trip with unchanged observed content hashes.
A successful resolution or compilation alone proves none of those behaviors.
No release baseline, bootstrap, SDK consumer execution or full upstream
compatibility is claimed by this decision record.
