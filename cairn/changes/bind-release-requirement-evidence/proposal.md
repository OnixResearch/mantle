# Proposal: Bind release requirement evidence

## Summary

Bind Mantle release requirement coverage to typed Valence references, a content-bound Cairn registry, and exact source or test evidence identities.

## Motivation

Mantle release provenance currently carries requirement IDs as strings. The default traceability profile also scans one curated bridge file that names implementation and test paths in comments.

These links are useful review indexes, but they do not mechanically establish registry membership, specification freshness, or current evidence-file content.

## Scope

- Add versioned typed requirement coverage to the release-evidence core.
- Bind coverage to exact Cairn registry and Valence reference identities.
- Add a machine-readable evidence manifest with BLAKE3-bound source, test, span, symbol, role, and receipt references.
- Add strict release policy that rejects stale, absent, duplicated, or wrong-repository coverage.
- Preserve current string IDs and bridge files as explicit compatibility inputs.

## Affected specs

- `release-provenance`: typed requirement coverage, evidence-manifest linkage, and compatibility boundaries.

## Dependencies

This change consumes exact reviewed outputs from Cairn `export-content-bound-requirement-registry` and Valence `adopt-content-bound-requirement-references`.

Mantle uses a small compatibility DTO in `crunch-release-core`. It does not add the std-oriented Valence crate to the no-std core.

## Non-goals

- Do not claim requirement satisfaction from registry membership or marker coverage.
- Do not make Mantle parse Rust semantics or execute tests during release verification.
- Do not rewrite existing release manifest identities.
- Do not remove the human-readable traceability bridge during migration.
- Do not transfer Cairn, Valence, Octet, verifier, or runtime authority to Mantle.

## Completion evidence

The change is complete when strict release verification rejects stale coverage, exact evidence artifacts are hash-bound, legacy bundles remain bounded, and cross-repository fixtures pass.
