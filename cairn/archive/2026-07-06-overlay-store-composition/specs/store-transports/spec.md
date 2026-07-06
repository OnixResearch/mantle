# Store Transports Specification

## Purpose

Defines the `store-transports` capability, including overlay store
composition: a writable overlay layered over read-only base store(s).

## Requirements

### Requirement: Mantle composes a writable overlay over read-only base stores [r[store_transports.overlay_composition]]

Mantle MUST support overlay store composition where a writable overlay store
is layered over one or more read-only base stores. Both the overlay and every
base MUST share the same logical store prefix. Read requests MUST consult the
overlay first and, on a miss, consult the base stack in declaration order
without copying base content into the overlay. All writes MUST route to the
overlay only; the base MUST be opened read-only and MUST reject writes.

#### Scenario: Read-through base hit does not mutate overlay [r[store_transports.overlay_composition.scenario.read-through-no-backfill]]

- GIVEN an overlay is composed over a base and the base has a blob, directory, or PathInfo that the overlay lacks
- WHEN Mantle reads that digest through the composed handle
- THEN Mantle MUST return the value from the base
- AND Mantle MUST NOT write that value into the overlay's blob, directory, or PathInfo service.

#### Scenario: Overlay shadows base [r[store_transports.overlay_composition.scenario.shadow]]

- GIVEN an overlay is composed over a base and both layers have a PathInfo for the same store path
- WHEN Mantle reads that store path through the composed handle
- THEN Mantle MUST return the overlay's PathInfo
- AND Mantle MUST NOT consult the base for that path.

#### Scenario: Writes route to overlay only [r[store_transports.overlay_composition.scenario.write-routing]]

- GIVEN an overlay is composed over a read-only base
- WHEN Mantle writes a blob, directory, PathInfo, or signed output through the composed handle
- THEN Mantle MUST persist the write into the overlay
- AND Mantle MUST NOT mutate any base store.

#### Scenario: Prefix mismatch is a hard error [r[store_transports.overlay_composition.scenario.prefix-mismatch]]

- GIVEN an overlay is configured with a base whose declared logical store prefix differs from the overlay's
- WHEN Mantle opens the composed handle
- THEN Mantle MUST fail before any read or write
- AND diagnostics MUST name both prefixes.

### Requirement: Overlay trust provenance is tracked per layer [r[store_transports.overlay_provenance_layer]]

Mantle MUST record which store layer produced or served each consumed path,
blob, directory, and PathInfo during overlay composition. A path served from
the base MUST inherit base trust; a path present in the overlay MUST be
governed by the overlay's signatures and MUST NOT inherit base signature
trust for that path. Artifact and closure attestations synthesized under
overlay composition MUST record the producing layer so verification can
distinguish base-sourced from overlay-sourced evidence.

#### Scenario: Base-sourced path inherits base trust [r[store_transports.overlay_provenance_layer.scenario.base-trust]]

- GIVEN an overlay is composed over a base and a path is read through the base because the overlay lacks it
- WHEN Mantle synthesizes or verifies an artifact attestation for that path
- THEN the attestation provenance MUST record the base as the producing layer
- AND verification MUST apply the base's trust policy to that path.

#### Scenario: Shadowed overlay path does not inherit base trust [r[store_transports.overlay_provenance_layer.scenario.no-inherited-trust]]

- GIVEN an overlay and base both have a PathInfo for the same store path and the overlay PathInfo is unsigned
- WHEN Mantle verifies that path under overlay composition
- THEN Mantle MUST treat the path as unsigned
- AND Mantle MUST NOT accept the base's signature as covering the overlay's content.

### Requirement: Overlay GC respects cross-layer references [r[store_transports.overlay_gc_cross_layer]]

Mantle's overlay garbage collection MUST NOT remove a path whose content is
only reachable through a base reference. The overlay reachability set MUST
include paths referenced by overlay PathInfo even when those references
resolve through the base. Mantle MUST NOT garbage-collect any base store; the
base is owned externally and opened read-only.

#### Scenario: Base-referenced path survives overlay GC [r[store_transports.overlay_gc_cross_layer.scenario.base-ref-survives]]

- GIVEN an overlay PathInfo references a store path whose content lives only in the base
- WHEN Mantle runs overlay garbage collection
- THEN Mantle MUST mark the referencing overlay path as live
- AND Mantle MUST NOT remove any content from the base.

#### Scenario: Base is never garbage-collected [r[store_transports.overlay_gc_cross_layer.scenario.no-base-gc]]

- GIVEN an overlay is composed over a base
- WHEN Mantle runs overlay garbage collection
- THEN Mantle MUST NOT open the base for mutation
- AND Mantle MUST NOT remove, rewrite, or delete any base blob, directory, PathInfo, or attestation file.

### Requirement: Overlay composition is declared via CLI [r[store_transports.overlay_cli_declaration]]

Mantle MUST accept a repeatable, ordered `--base-store <state-dir>` option
that stacks one or more read-only base stores below the writable local store.
The local store remains the overlay. Declared bases MUST stack in declaration
order, with the overlay consulted first and bases consulted in order on
misses. The option MUST fail closed when a declared base state directory does
not exist or cannot be opened read-only.

#### Scenario: Multiple bases stack in declaration order [r[store_transports.overlay_cli_declaration.scenario.ordered-stack]]

- GIVEN an overlay is composed over two bases declared as `--base-store A --base-store B`
- WHEN Mantle reads a digest missing from the overlay
- THEN Mantle MUST consult base A before base B
- AND a hit in base A MUST prevent consulting base B.

#### Scenario: Missing base state directory fails closed [r[store_transports.overlay_cli_declaration.scenario.missing-base-fails]]

- GIVEN an overlay is configured with a `--base-store` path that does not exist
- WHEN Mantle opens the composed handle
- THEN Mantle MUST fail before any build or store operation
- AND diagnostics MUST name the missing base path.
