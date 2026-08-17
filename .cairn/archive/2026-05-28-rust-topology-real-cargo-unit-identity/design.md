# Design: Real selected Cargo unit identity for native artifacts

## Context

Cargo unit graph dependency edges point at specific unit indices. Mantle's prior fix lowered that edge into a synthetic variant key based on package/crate/kind/mode. That key was reviewable, but not injective over Cargo-selected units.

## Decisions

### 1. Preserve Cargo unit IDs from unit graph entries

**Choice:** Build producer identity with `rust_unit_id_from_unit_value(index, unit)` where `index` is the selected Cargo unit graph index referenced by a dependency edge.

**Rationale:** Cargo dependency edges already identify exact producer units by index. Using that entry preserves duplicate same-package units with different features, rustc metadata, source facts, or dependency closures.

### 2. Lower native target units from Cargo oracle derivations

**Choice:** Build native target units from the Cargo unit derivation graph for supported target units, then add only supplemental missing library units for transitive dependency packages absent from selected targets.

**Rationale:** Regenerating native units from package manifests loses Cargo's selected unit partitioning. Oracle-derived units keep the same unit IDs and dependency producer IDs that execution later consumes.

### 3. Constrain deduplication to exact unit IDs

**Choice:** Remove package/crate/kind/mode deduplication. Only normalize duplicate records with the same unit ID, preferring absolute source paths when the same selected unit is represented twice.

**Rationale:** Exact unit-ID normalization handles relative/absolute duplicate descriptions without collapsing distinct Cargo-selected producer variants.

### 4. Match package-only fallback by crate name

**Choice:** For legacy package-only placeholders, search candidates match both package ID and dependency crate name.

**Rationale:** A package can produce both a `lib` and a `build_script_build` host artifact. Package-only fallback should fail on true same-crate ambiguity, not unrelated same-package outputs.

## Risks / Trade-offs

- Unit IDs remain tied to Cargo unit graph ordering while Cargo is the oracle. That is acceptable because this phase explicitly retains Cargo unit graph material as reviewed input.
- Supplemental lib units are still a bounded compatibility bridge for dependency packages missing from selected target units; they must not replace real selected Cargo producer units.
