# Nix evidence core

Mantle normalizes Nix-related evidence into an identity-only DTO before release
or downstream handoff code interprets it. The core validates:

- store path prefix and Nix-style store hash/name shape;
- derivation and output identity;
- realization role;
- artifact BLAKE3 digest versus measured digest;
- explicit caveats; and
- non-claims.

`nix_evidence_core::validate_nix_evidence` is pure and works on already-loaded
rows. Adapters for Mantle build reports, release provenance rows, Cairn Nix
gates, Molten promotion evidence, and Valence provenance inputs should measure
or parse their source material in shell code, normalize into these rows, and then
call the core.

Passing validation proves only that the row describes realization identity with
bounded caveats. It does not prove build correctness, evaluation correctness,
release correctness, semantic correctness, or substituter trust.
