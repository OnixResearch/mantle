# Design: Unit-variant-aware native Rust artifact binding

## Context

Prior fixes added deterministic crate metadata and then preserved transitive search paths. Those were necessary but not sufficient because package ID is still too coarse for artifact identity. Cargo can build multiple units for one package when features, host/target role, build-script metadata, or selected target shape differ. Rustc metadata can then require one exact variant while another same-package variant has the same crate name.

## Design

Introduce an artifact identity that can represent the selected producer unit variant. A suitable key is derived from reviewable unit facts already present in `RustUnitDerivationSummary`, such as:

- producer `unit_id`,
- package ID,
- target name and kind,
- mode/profile,
- source digest,
- selected features / rustc metadata disambiguator.

Use that identity for two separate concerns:

1. **Direct binding**: rewrite `artifact:<...>` placeholders to the exact selected producer output for the consumer dependency edge.
2. **Search scope**: append `-L dependency` paths from the selected producer's transitive closure, not from every historical artifact for the same package or crate name.

The planner should preserve the Cargo-selected producer edge when lowering dependency artifacts. If an old package-only placeholder is still encountered at execution time and multiple candidate producer variants exist, execution must fail closed before rustc instead of guessing.

## Functional core / shell split

Pure helpers should compute:

- candidate producer variants for a dependency,
- ambiguity/missing-producer blockers,
- selected direct artifact bindings,
- selected transitive search-path closure with deduplication.

The shell should only read/write receipts, execute rustc, and thread produced artifact paths into the pure helpers.

## Non-goals

This does not claim complete Cargo parity for every target kind. It narrows the current native topology frontier by preserving unit-variant identity for already-supported lib/proc-macro/custom-build topology execution.
