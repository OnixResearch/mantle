# Design: Exact selected host unit identity

## Context

Cargo can select multiple host units for one package target shape when features, metadata, or dependency graph position differs. Mantle currently uses one map entry per package/name/kind, so later entries can overwrite earlier selected host units.

## Decisions

### 1. Represent selected host units as a multimap or per-entry list

**Choice:** Replace `selected_host_unit_ids_by_key` with data that preserves all selected host entries, including unit ID plus discriminating facts needed to bind consumers.

**Rationale:** A single map value cannot represent Cargo's selected unit multiplicity.

### 2. Bind host artifacts from exact Cargo edges when available

**Choice:** Prefer host artifact producer IDs from Cargo dependency edges. Use package/name/kind lookup only as a fail-closed legacy fallback.

**Rationale:** Cargo edges are the authoritative selected producer relation while Cargo remains the oracle.

### 3. Fail closed on ambiguous host fallback

**Choice:** If a fallback lookup sees multiple host candidates for a package/name/kind, emit a deterministic ambiguity blocker before rustc.

**Rationale:** Guessing recreates the identity bug.

## Risks / Trade-offs

- Some legacy fixtures may need explicit producer identity added.
- Host graph receipts may grow because duplicate host units are no longer collapsed.
