# Proposal: Preserve real Cargo unit identity in native Rust artifacts

## Problem

The archived unit-variant artifact change stopped some package-ID overwrites, but done-review found the producer identity was still too coarse. It derived pseudo variant keys from package/crate/kind/mode and deduplicated by those keys. That can collapse two Cargo-selected units with the same package, crate name, target kind, and mode but different selected features, metadata, or dependency facts.

The clean self-probe succeeded only after that coarse deduplication, so it did not prove the `rust_package_planning.native_unit_variant_artifacts` requirement honestly.

## Change

Use the selected Cargo unit graph entry as the producer identity for native Rust dependency and host artifacts. Preserve duplicate selected units in native target planning instead of regenerating or deduplicating them by package-level facts. Allow only exact same `unit_id` normalization when duplicate relative/absolute descriptions of the same Cargo unit appear.

Package-only fallback remains fail-closed, but candidate selection must match package ID plus crate name so build-script artifacts do not become ambiguous just because the same package also produced a library.

## Success criteria

- `dependency_producer_unit_id()` and native dependency lowering use selected Cargo unit identity, not package/crate/kind/mode pseudo keys.
- Native unit graph planning preserves duplicate selected Cargo target units.
- Exact duplicate normalization cannot collapse distinct selected Cargo unit IDs.
- Focused tests exercise production lowering from representative Cargo unit JSON.
- Dirty self-probe reaches success with no `crunch_store` / `snix_castore` blocker.
