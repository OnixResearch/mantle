# Preserves release evidence carriers

Mantle treats Preserves payloads as release-evidence carrier bytes, not as native
proof of release correctness. A carrier row can be either opaque or
adapter-backed:

- opaque rows bind the Preserves payload and canonical byte digests and keep the
  payload semantics outside Mantle;
- adapter-backed rows additionally bind an adapter id, adapter digest, and
  bounded adapter facts that explain how Mantle projects selected metadata.

`preserves_release_carrier::validate_preserves_release_carriers` is the pure
validation core for already-loaded carrier rows. The release shell remains
responsible for reading files, computing BLAKE3 digests, and deciding where rows
come from.

## Required non-claims

Every row must state that the carrier is opaque and that validation is not a
release-correctness claim. Passing validation proves only that Mantle received a
supported Preserves carrier row with matching digest shape, supported role,
schema id, adapter shape, and bounded non-claims. It does not prove release
correctness, artifact correctness, deployment safety, full reproducibility, or
semantic correctness.
